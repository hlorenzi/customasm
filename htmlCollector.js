const process = require('process');
const parser = require('node-html-parser');
const minify = import('minify');
const path = require('path');
const fs = require('fs');

async function get(link, from = null, buffer = false) {
    const p = from === null ? './' : path.dirname(path.resolve(from));
    if (link.startsWith('./')) {
        return [
            path.join(p, link),
            buffer
                ? fs.readFileSync(path.join(p, link))
                : fs.readFileSync(path.join(p, link)).toString(),
        ];
    } else {
        return [link, await (await fetch(link)).text()];
    }
}

String.prototype.replaceBetween = function (start, end, value) {
    return this.substring(0, start) + value + this.substring(end);
};

function* match(r, input) {
    const regex = cloneRegExp(r);
    let m;

    while ((m = regex.exec(input)) !== null) {
        // This is necessary to avoid infinite loops with zero-width matches
        if (m.index === regex.lastIndex) {
            regex.lastIndex++;
        }

        yield m;
    }
}

function cloneRegExp(regex) {
    var pattern = regex.source;
    var flags = '';

    regex.global && (flags += 'g');
    regex.ignoreCase && (flags += 'i');
    regex.multiline && (flags += 'm');
    regex.dotAll && (flags += 's');
    regex.sticky && (flags += 'y');
    regex.unicode && (flags += 'u');

    return new RegExp(pattern, flags);
}

const languages = {
    javascript: {
        comments: [/\/\/(.*?)\r?\n/g, /\/\*(.*?)\*\//gs],
    },
    css: {
        comments: [/\/\*(.*?)\*\//gs],
    },
    python: {
        comments: [/\#(.*?)\r?\n/g],
    },
    html: {
        comments: [/\<\!\-\-(.*?)\-\-\>/gs],
    },
};

const files = new Map();

async function CODE([file, s], language) {
    if (files.has(file)) return files.get(file);
    files.set(file, undefined);
    const functions = new Map();
    const collections = [];
    let open = null;
    languages[language].comments
        .flatMap((regex) => [...match(regex, s)])
        .sort((m1, m2) => m1.index - m2.index)
        .forEach((m) => {
            if (m[1].startsWith('?')) {
                const WORDS = [
                    ...match(
                        /\(.*?\)|\`.*?\`|\"[^\n]*?\"|\'[^\n]*?\'|\S+/gs,
                        m[1].substring(1)
                    ),
                ].map((a) => a[0]);

                if (WORDS[0] === 'COLLECTION') {
                    if (WORDS[1] === 'FUNCTION') {
                        const t = /\(\s*([A-Z]+)\s*\-\>\s*([A-Z]+)\s*\)/.exec(
                            WORDS[3]
                        );
                        if (WORDS[4] !== ':')
                            throw SyntaxError(
                                `Unexpected ${WORDS[4] || 'EOC'} expecting :`
                            );
                        functions.set(/\'(.*?)\'/.exec(WORDS[2])[1], {
                            expects: t[1],
                            returns: t[2],
                            function: eval(/\`(.*?)\`/s.exec(WORDS[5])[1]),
                        });
                    }
                    if (WORDS[1] === 'FROM') {
                        if (open !== null)
                            throw SyntaxError(`COLLECTION never closed`);
                        if (WORDS[3] !== 'AS')
                            throw SyntaxError(
                                `Unexpected ${WORDS[3] || 'EOC'} expecting AS`
                            );
                        const WITH = [];
                        let x = 7;
                        if (WORDS[5] === 'WITH') {
                            WITH.push(/\'(.*?)\'/.exec(WORDS[6])[1]);
                            while (WORDS[x] === 'AND') {
                                WITH.push(/\'(.*?)\'/.exec(WORDS[x + 1])[1]);
                                x += 2;
                            }
                        }
                        if (WORDS[x] !== '[')
                            throw SyntaxError(
                                `Unexpected ${WORDS[x] || 'EOC'} expecting [`
                            );
                        open = {
                            from: /\"(.*?)\"/.exec(WORDS[2])[1],
                            as: /\((.*?)\)/s.exec(WORDS[4])[1],
                            with: WITH,
                            start: m.index + m[0].length,
                        };
                    }
                } else if (WORDS[0] === ']') {
                    collections.push({
                        ...open,
                        end: m.index,
                    });
                    open = null;
                }
            }
        });
    if (open !== null) throw SyntaxError(`COLLECTION never closed`);

    for (var collection of collections) {
        let str = '';
        let lang = /CODE\{(\w+)\}/.exec(collection.as);
        if (lang !== null && lang[1] === 'html')
            str = await HTML(await get(collection.from, file));
        else if (lang !== null)
            str = await CODE(await get(collection.from, file), lang[1]);
        else if (collection.as === 'BUFFER')
            str = (await get(collection.from, file, true))[1];
        else if (collection.as === 'STRING')
            str = (await get(collection.from, file))[1];

        str = collection.with.reduce(
            (acc, item) => functions.get(item).function(acc),
            str
        );

        s = s.replaceBetween(collection.start, collection.end, str);
    }

    files.set(file, s);
    return s;
}

async function HTML([file, s]) {
    if (files.has(file)) return files.get(file);
    s = await CODE([file, s], 'html');

    const html = parser.parse(s);

    for (var link of html.getElementsByTagName('link')) {
        if (link.hasAttribute('href') && !link.hasAttribute('always-keep')) {
            href = link.getAttribute('href');
            link.removeAttribute('href');
            link.removeAttribute('rel');
            link.removeAttribute('type');
            link.tagName = 'style';
            link.innerHTML = await CODE(await get(href, file), 'css');
        }
    }

    for (var script of html.getElementsByTagName('script')) {
        if (script.hasAttribute('src') && !script.hasAttribute('always-keep')) {
            src = script.getAttribute('src');
            script.removeAttribute('src');
            script.removeAttribute('crossorigin');
            script.removeAttribute('integrity');
            script.innerHTML = await CODE(await get(src, file), 'javascript');
        }
    }

    return html.toString();
}

get(process.argv[2])
    .then(HTML)
    .then((s) =>
        Promise.all([
            minify,
            new Promise((r) => fs.writeFile(process.argv[3], s, r)),
        ])
    )
    .then(([a]) =>
        a
            .minify(process.argv[3])
            .then((s) => fs.writeFileSync(process.argv[3], s))
    );
