let g_wasm = null
let g_codeEditor = null


function main()
{
	setupEditor()
	onResize()
	
	document.body.onresize = onResize
	window.onresize = onResize
	window.onkeydown = onKeyDown
	window.onbeforeunload = onBeforeUnload
	
	fetch("customasm.gc.wasm")
		.then(r => r.arrayBuffer())
		.then(r => WebAssembly.instantiate(r))
		.then(wasm =>
		{
			g_wasm = wasm
			document.getElementById("buttonAssemble").disabled = false
		})
}


function setupEditor()
{
	g_codeEditor = CodeMirror(document.getElementById("divCodeMirror"),
	{
		lineNumbers: true, matchBrackets: true, indentWithTabs: true, highlightSelectionMatches : true,
		tabSize: 4, indentUnit: 4, mode: "z80"
	})
	
	fetch("../examples/test.asm")
		.then(r => r.text())
		.then(r => g_codeEditor.setValue(r))
	
	g_codeEditor.refresh()
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
	output = output.replace(/\x1b\[0m\x1b\[90m/g, "</span><span style='color:gray;'>")
	output = output.replace(/\x1b\[0m\x1b\[91m/g, "</span><span style='color:red;'>")
	output = output.replace(/\x1b\[0m\x1b\[93m/g, "</span><span style='color:#f80;'>")
	output = output.replace(/\x1b\[0m\x1b\[97m/g, "</span><span style='color:black;'>")
	output = output.replace(/\x1b\[0m/g, "</span><span style='color:black;'>")
	
	output = "<span style='color:black;'>" + output + "</span>"
	
	let isError = output.includes("error")
	
	let divText = document.getElementById("divOutputText")
	divText.innerHTML = output
	divText.style.whiteSpace = (isError || format <= 1) ? "pre" : "normal"
}


function makeRustString(str)
{
	//console.log("makeRustString")
	//console.log(str)
	
	let bytes = new TextEncoder("utf-8").encode(str)
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
	
	let str = new TextDecoder("utf-8").decode(new Uint8Array(bytes))
	//console.log(str)
	return str
}


function dropRustString(ptr)
{
	//console.log("dropRustString")
	//console.log(ptr)
	
	g_wasm.instance.exports.wasm_string_drop(ptr)
}