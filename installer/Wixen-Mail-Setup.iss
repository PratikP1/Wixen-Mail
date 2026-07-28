; Wixen Mail setup.
;
; Build it with scripts/build-installer.sh rather than running ISCC by hand.
; The version comes in from Cargo.toml, and this file refuses to compile
; without it: an installer labelled with a number somebody forgot to change is
; worse than one that did not build. The previous version of this file said
; 0.1.0-beta.1 while the crate said 0.1.0-alpha.14.

#ifndef AppVersion
  #error Run scripts/build-installer.sh, which passes /DAppVersion and /DVersionInfo
#endif
#ifndef VersionInfo
  #error Run scripts/build-installer.sh, which passes /DAppVersion and /DVersionInfo
#endif

#define AppName "Wixen Mail"
#define Publisher "Pratik Patel"
#define RepoUrl "https://github.com/PratikP1/Wixen-Mail"

[Setup]
; Fixed for the life of the application. Without it Inno keys the install on
; the display name, so renaming the program would strand the old one in Apps
; and Features with no way to remove it.
AppId={{9C2E6B41-0F7D-4A83-B5E1-27D4A9F3C608}
AppName={#AppName}
AppVersion={#AppVersion}
AppVerName={#AppName} {#AppVersion}
VersionInfoVersion={#VersionInfo}
AppPublisher={#Publisher}
AppPublisherURL={#RepoUrl}
AppSupportURL={#RepoUrl}/issues
AppUpdatesURL={#RepoUrl}/releases
LicenseFile=..\LICENSE

; Installing for one user needs no administrator rights, so there is no
; elevation prompt and no switch to the secure desktop part way through setup.
; Whoever is installing chooses on the first page; /ALLUSERS and /CURRENTUSER
; answer it from a command line for an unattended install.
PrivilegesRequired=lowest
PrivilegesRequiredOverridesAllowed=dialog commandline

DefaultDirName={autopf}\{#AppName}
DefaultGroupName={#AppName}
DisableProgramGroupPage=yes
UninstallDisplayName={#AppName}
UninstallDisplayIcon={app}\wixen-mail.exe

; Ask a running copy to close rather than failing on a locked file, and leave
; it closed afterwards rather than reopening a window nobody asked for.
CloseApplications=yes
RestartApplications=no

; "compatible" rather than "os": Windows on ARM runs x64 programs under
; emulation, and the stricter spelling would refuse to install on a Surface for
; no reason anybody could act on. Needs Inno 6.3 or newer.
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible

WizardStyle=modern
SetupLogging=yes
Compression=lzma2
SolidCompression=yes
OutputDir=..\dist
OutputBaseFilename=Wixen-Mail-Setup-{#AppVersion}

[Languages]
Name: "english"; MessagesFile: "compiler:Default.isl"

[Tasks]
Name: "desktopicon"; Description: "Create a shortcut on the desktop"; GroupDescription: "Shortcuts:"

[Files]
Source: "..\target\release\wixen-mail.exe"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\LICENSE"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\README.md"; DestDir: "{app}"; Flags: ignoreversion
; The guides somebody needs while the application is not working: setting up a
; provider, the keyboard shortcuts, and what to try when mail will not arrive.
; Not recursive, because the folders below docs are notes to ourselves.
Source: "..\docs\*.md"; DestDir: "{app}\docs"; Flags: ignoreversion

[Icons]
Name: "{group}\{#AppName}"; Filename: "{app}\wixen-mail.exe"
Name: "{autodesktop}\{#AppName}"; Filename: "{app}\wixen-mail.exe"; Tasks: desktopicon

[Run]
Filename: "{app}\wixen-mail.exe"; Description: "Start {#AppName}"; Flags: nowait postinstall skipifsilent

[UninstallRun]
; Uninstalling removes everything, and most of everything is out of an
; uninstaller's reach: the OAuth tokens and the master key are in the Windows
; credential store, and the data folder moves when WIXEN_MAIL_DATA says so. The
; program knows where both are, so it clears them before the files go.
;
; runascurrentuser is the part that matters, and it is the flag this section
; takes rather than the runasoriginaluser that [Run] takes. Elevated, this would
; clear the administrator's credential store and leave the tokens belonging to
; the person whose mail it is exactly where they were.
Filename: "{app}\wixen-mail.exe"; Parameters: "--erase-all-data"; RunOnceId: "EraseData"; Flags: runhidden waituntilterminated runascurrentuser

[UninstallDelete]
; Belt and braces for the default location, for the case where the step above
; could not run at all.
Type: filesandordirs; Name: "{localappdata}\wixen-mail"
