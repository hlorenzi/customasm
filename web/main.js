let g_wasm = null
let g_codeEditor = null


function main()
{
	setupEditor()
	onResize()
	
	document.body.onresize = onResize
	window.onkeydown = onKeyDown
	window.onbeforeunload = onBeforeUnload
	
	fetch("customasm.gc.wasm")
		.then(r => r.arrayBuffer())
		.then(r => WebAssembly.instantiate(r))
		.then(wasm =>
		{
			g_wasm = wasm
			document.getElementById("buttonAssemble").disabled = false
			setupVersionString()
		})
}


function setupEditor()
{
	g_codeEditor = CodeMirror(document.getElementById("divCodeMirror"),
	{
		lineNumbers: true, matchBrackets: true, indentWithTabs: true, highlightSelectionMatches : true,
		tabSize: 4, indentUnit: 4, mode: "z80"
	})
	
	fetch("../examples/basic.asm")
		.then(r => r.text())
		.then(r => g_codeEditor.setValue(r))
	
	g_codeEditor.refresh()
}


function setupVersionString()
{
	let outputPtr = null
	try
	{
		outputPtr = g_wasm.instance.exports.wasm_get_version()
	}
	catch (e)
	{
		throw e
	}
	
	let output = readRustString(outputPtr)
	dropRustString(outputPtr)
	
	document.getElementById("spanVersion").innerHTML = output
}


function onResize()
{
	let rectInput = document.getElementById("divCodeMirror").getBoundingClientRect()
	g_codeEditor.setSize(rectInput.width, rectInput.height)
}


function onBeforeUnload()
{
	return "Your work will be lost if you close the page."
}


function onKeyDown(ev)
{
	if (!ev.ctrlKey)
		return
	
	if (ev.key == "Enter")
	{
		ev.preventDefault()
		assemble()
	}
}


function assemble()
{
	if (g_wasm == null)
		return
	
	let format = document.getElementById("selectFormat").selectedIndex
	
	let asmPtr = makeRustString(g_codeEditor.getValue())
	let outputPtr = null
	try
	{
		outputPtr = g_wasm.instance.exports.wasm_assemble(format, asmPtr)
	}
	catch (e)
	{
		alert("Error assembling!\n\n" + e)
		throw e
	}
	
	let output = readRustString(outputPtr)
	
	dropRustString(asmPtr)
	dropRustString(outputPtr)
	
	output = output.replace(/\n/g, "<br>")
	output = output.replace(
		/ --> asm:\x1b\[0m\x1b\[90m(\d+):(\d+)/g,
		(_, line, column) =>
			` --> asm:<button class="a" onclick="g_codeEditor.focus();g_codeEditor.setCursor({line:${line - 1
			},ch:${column - 1}})">${line}:${column}</button>`
	);
	output = output.replace(/\x1b\[90m/g, "</span><span style='color:gray;'>")
	output = output.replace(/\x1b\[91m/g, "</span><span style='color:red;'>")
	output = output.replace(/\x1b\[93m/g, "</span><span style='color:#f80;'>")
	output = output.replace(/\x1b\[96m/g, "</span><span style='color:#08f;'>")
	output = output.replace(/\x1b\[97m/g, "</span><span style='color:black;'>")
	output = output.replace(/\x1b\[1m/g, "</span><span style='font-weight:bold;'>")
	output = output.replace(/\x1b\[0m/g, "</span><span style='color:black;'>")
	//</span>
	
	output = "<span style='color:black;'>" + output + "</span>"
	
	let divText = document.getElementById("divOutputText")
	divText.innerHTML = output
	divText.style.whiteSpace = "no-wrap"
}


function makeRustString(str)
{
	//console.log("makeRustString")
	//console.log(str)
	
	let bytes = window.TextEncoder ? new TextEncoder("utf-8").encode(str) : stringToUtf8ByteArray(str)
	//console.log(bytes)
	
	let ptr = g_wasm.instance.exports.wasm_string_new(bytes.length)
	
	for (let i = 0; i < bytes.length; i++)
		g_wasm.instance.exports.wasm_string_set_byte(ptr, i, bytes[i])
	
	//console.log(ptr)
	return ptr
}


function readRustString(ptr)
{
	//console.log("readRustString")
	//console.log(ptr)
	
	let len = g_wasm.instance.exports.wasm_string_get_len(ptr)
	//console.log(len)
	
	let bytes = []
	for (let i = 0; i < len; i++)
		bytes.push(g_wasm.instance.exports.wasm_string_get_byte(ptr, i))
	
	//console.log(bytes)
	
	let str = window.TextDecoder ? new TextDecoder("utf-8").decode(new Uint8Array(bytes)) : utf8ByteArrayToString(bytes)
	//console.log(str)
	return str
}


function dropRustString(ptr)
{
	//console.log("dropRustString")
	//console.log(ptr)
	
	g_wasm.instance.exports.wasm_string_drop(ptr)
}


// From https://github.com/google/closure-library/blob/e877b1eac410c0d842bcda118689759512e0e26f/closure/goog/crypt/crypt.js#L115
function stringToUtf8ByteArray(str)
{
	let out = [], p = 0
	for (let i = 0; i < str.length; i++) {
		let c = str.charCodeAt(i)
		if (c < 128) {
			out[p++] = c
		} else if (c < 2048) {
			out[p++] = (c >> 6) | 192
			out[p++] = (c & 63) | 128
		} else if (
			((c & 0xFC00) == 0xD800) && (i + 1) < str.length &&
			((str.charCodeAt(i + 1) & 0xFC00) == 0xDC00)) {
			// Surrogate Pair
			c = 0x10000 + ((c & 0x03FF) << 10) + (str.charCodeAt(++i) & 0x03FF)
			out[p++] = (c >> 18) | 240
			out[p++] = ((c >> 12) & 63) | 128
			out[p++] = ((c >> 6) & 63) | 128
			out[p++] = (c & 63) | 128
		} else {
			out[p++] = (c >> 12) | 224
			out[p++] = ((c >> 6) & 63) | 128
			out[p++] = (c & 63) | 128
		}
	}
	return out
}


// From https://github.com/google/closure-library/blob/e877b1eac410c0d842bcda118689759512e0e26f/closure/goog/crypt/crypt.js#L149
function utf8ByteArrayToString(bytes)
{
	let out = [], pos = 0, c = 0
	while (pos < bytes.length) {
		let c1 = bytes[pos++]
		if (c1 < 128) {
			out[c++] = String.fromCharCode(c1)
		} else if (c1 > 191 && c1 < 224) {
			let c2 = bytes[pos++]
			out[c++] = String.fromCharCode((c1 & 31) << 6 | c2 & 63)
		} else if (c1 > 239 && c1 < 365) {
			// Surrogate Pair
			let c2 = bytes[pos++]
			let c3 = bytes[pos++]
			let c4 = bytes[pos++]
			let u = ((c1 & 7) << 18 | (c2 & 63) << 12 | (c3 & 63) << 6 | c4 & 63) - 0x10000
			out[c++] = String.fromCharCode(0xD800 + (u >> 10))
			out[c++] = String.fromCharCode(0xDC00 + (u & 1023))
		} else {
			let c2 = bytes[pos++]
			let c3 = bytes[pos++]
			out[c++] =
				String.fromCharCode((c1 & 15) << 12 | (c2 & 63) << 6 | c3 & 63)
		}
	}
	return out.join('')
}