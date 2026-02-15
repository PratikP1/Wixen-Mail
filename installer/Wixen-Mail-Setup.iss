[Setup]
AppName=Wixen Mail
AppVersion=0.1.0-beta.1
DefaultDirName={autopf}\Wixen Mail
DefaultGroupName=Wixen Mail
OutputDir=dist
OutputBaseFilename=Wixen-Mail-Setup
Compression=lzma
SolidCompression=yes

[Files]
Source: "..\target\release\wixen-mail.exe"; DestDir: "{app}"; Flags: ignoreversion

[Icons]
Name: "{group}\Wixen Mail"; Filename: "{app}\wixen-mail.exe"
Name: "{autodesktop}\Wixen Mail"; Filename: "{app}\wixen-mail.exe"; Tasks: desktopicon

[Tasks]
Name: "desktopicon"; Description: "Create a desktop icon"; GroupDescription: "Additional icons:"
