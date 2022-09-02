#!/usr/bin/python3

const process = require('process');
const parser = require('node-html-parser');
const minify = import('minify');
const path = require('path');
const fs = require('fs');

const file = process.argv[2];

const p = path.dirname(path.resolve(file));

const html = parser.parse(fs.readFileSync(file));

function get(link) {
    if (link.startsWith('./')) {
        return fs.readFileSync(path.join(p, link)).toString();
    } else {
        return requests.get(link).text;
    }
}

for (var link of html.getElementsByTagName('link')) {
    if (link.hasAttribute('href') && !link.hasAttribute('always-keep')) {
        href = link.getAttribute('href');
        link.removeAttribute('href');
        link.removeAttribute('rel');
        link.removeAttribute('type');
        link.tagName = 'style';
        link.innerHTML = get(href);
    }
}

for (var script of html.getElementsByTagName('script')) {
    if (script.hasAttribute('src') && !script.hasAttribute('always-keep')) {
        src = script.getAttribute('src');
        script.removeAttribute('src');
        script.removeAttribute('crossorigin');
        script.removeAttribute('integrity');
        script.innerHTML = get(src);
    }
}

const out = process.argv[3];

fs.writeFileSync(out, html.toString());
minify.then((a) => a.minify(out).then((r) => fs.writeFileSync(out, r)));
