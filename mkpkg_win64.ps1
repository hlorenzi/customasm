$pkgTempPath = "./.pkgtemp"
$pkgPath = "./pkg"

# Create empty temp folder
If (Test-Path $pkgTempPath)
	{ Remove-Item $pkgTempPath -Force -Recurse }
	
New-Item -ItemType Directory -Path $pkgTempPath | Out-Null

# Copy needed files
Copy-Item -Path "./target/release/customasm.exe" -Destination ($pkgTempPath + "/customasm.exe")
Copy-Item -Path "./README_PKG.txt" -Destination ($pkgTempPath + "/README.txt")
Copy-Item -Path "./examples" -Recurse -Destination ($pkgTempPath + "/examples")

# Strip debug data from binary to reduce file size
strip ($pkgTempPath + "/customasm.exe")

# Compress and place in correct output folder
If (!(Test-Path $pkgPath))
	{ New-Item -ItemType Directory -Path $pkgPath | Out-Null }
	
Compress-Archive -Path ($pkgTempPath + "/*") -Force -DestinationPath ($pkgPath + "/customasm.win64.zip")

# Delete temp folder
Remove-Item $pkgTempPath -Force -Recurse