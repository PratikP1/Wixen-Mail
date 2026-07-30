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
; The wizard's own icon, so setup is recognisable in the taskbar before the
; application it installs exists.
SetupIconFile=..\assets\icon.ico

; Ask a running copy to close rather than failing on a locked file, and leave
; it closed afterwards rather than reopening a window nobody asked for.
CloseApplications=yes
RestartApplications=no

; Uninstalling deletes the data folder and clears the credential store. Doing
; that under a running copy takes half of it, lets the copy still running write
; its settings back over the rest, and leaves the program neither installed nor
; removed. Setup and Uninstall both stop here and ask for it to be closed.
;
; The name is held by the running program. It comes from
; application::running::MUTEX_NAME, and a test in that module fails if the two
; stop matching, because otherwise the mistake is silent: Uninstall would look
; for a marker nobody holds, find it free, and go ahead.
AppMutex=WixenMail-Running

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

[Code]
// An install for one user and an install for everybody are two separate
// installations of the same program, and Windows does not tell either about the
// other. They land in different folders, write different uninstall entries, and
// each puts a shortcut called "Wixen Mail" in a different Start Menu, which
// Windows then merges into one search result. Whichever one comes back first is
// what launches.
//
// That happened. An install for one user was uninstalled, the uninstall did not
// finish, and its shortcut kept starting a build from three versions earlier
// while a current one sat in Program Files. Three bug reports came out of it,
// all of them against code that had already been replaced, and the only way to
// tell was the version in the log.
//
// So setup looks in the scope it is not installing into, and offers to remove
// what it finds there first.
//
// What it will not catch: an install for everybody is made by an elevated
// setup, and while it is elevated both HKEY_CURRENT_USER and {localappdata}
// belong to whoever answered the elevation prompt. Somebody installing for a
// different person than the one whose per-user copy is stale is looking in the
// wrong profile and will find nothing. That is the uncommon case and it is not
// solved here; the common one, where a person elevates their own account, is.

const
  UninstallEntry =
    'Software\Microsoft\Windows\CurrentVersion\Uninstall\' +
    '{9C2E6B41-0F7D-4A83-B5E1-27D4A9F3C608}_is1';

{ Where a copy installed in the other scope keeps its uninstaller, or ''. }
function UninstallerInTheOtherScope(): String;
var
  Root: Integer;
  Command: String;
begin
  Result := '';
  { Installing for everybody, so the other scope is this user's, and the other
    way round. }
  if IsAdminInstallMode() then
    Root := HKEY_CURRENT_USER
  else
    Root := HKEY_LOCAL_MACHINE;

  if RegQueryStringValue(Root, UninstallEntry, 'UninstallString', Command) then
    Result := RemoveQuotes(Command);
end;

{ An uninstaller left behind by an uninstall that did not finish, or ''.

  A second way of finding the same thing, for the case the first one misses. An
  uninstall that stops partway can stop anywhere: the incident this was written
  for left both the files and the registry entry, so the check above would have
  caught it, but an uninstall that got one step further would have taken the
  entry and left the folder and the shortcut. Then Windows knows about nothing
  and a shortcut still starts it, which is the worst of the two. }
function StrandedUninstaller(): String;
var
  Candidate: String;
begin
  Result := '';
  if IsAdminInstallMode() then
    Candidate := ExpandConstant('{localappdata}\Programs\{#AppName}\unins000.exe')
  else
    Candidate := ExpandConstant('{commonpf}\{#AppName}\unins000.exe');

  if FileExists(Candidate) then
    Result := Candidate;
end;

// Take the other copy's program files and shortcut away, and nothing else.
//
// Deliberately not by running its uninstaller, which would be the obvious way
// and is wrong. Both copies read and write the same data folder, and the
// uninstaller deletes it: it runs `--erase-all-data`, which clears the mail
// cache and every password and token in the Windows credential store, and then
// removes {localappdata}\wixen-mail for good measure. Tidying up an old copy
// would have silently emptied the mailbox of the new one.
//
// So this removes what causes the problem, which is a second program folder and
// a second Start Menu entry with the same name, and leaves the mail alone. The
// uninstall entry goes too, or Apps and Features keeps offering to remove
// something that is no longer there.
procedure RemoveTheOtherCopy(Folder: String);
var
  Root: Integer;
begin
  DelTree(Folder, True, True, True);

  if IsAdminInstallMode() then
    Root := HKEY_CURRENT_USER
  else
    Root := HKEY_LOCAL_MACHINE;
  RegDeleteKeyIncludingSubkeys(Root, UninstallEntry);

  { Both possible homes for the stale shortcut. Whichever this copy is not
    using is the one that had it. }
  if IsAdminInstallMode() then
    DelTree(ExpandConstant('{userprograms}\{#AppName}'), True, True, True)
  else
    DelTree(ExpandConstant('{commonprograms}\{#AppName}'), True, True, True);
end;

function PrepareToInstall(var NeedsRestart: Boolean): String;
var
  Uninstaller: String;
begin
  Result := '';

  Uninstaller := UninstallerInTheOtherScope();
  if Uninstaller = '' then
    Uninstaller := StrandedUninstaller();
  if Uninstaller = '' then
    Exit;

  { Asked rather than done, and the message says what will and will not happen
    to the mail. Somebody who has just been told a second copy exists has no way
    of knowing that removing it might empty their mailbox, so the answer to that
    is given before the question. }
  if MsgBox(
       'Another copy of Wixen Mail is installed on this computer, in a '
       + 'different place:' + #13#10#13#10
       + ExtractFileDir(Uninstaller) + #13#10#13#10
       + 'Two copies means two Start Menu entries with the same name, and '
       + 'Windows may start either one, so you can end up running the older '
       + 'version without knowing.' + #13#10#13#10
       + 'Remove the other copy now? Your mail, accounts and settings are '
       + 'shared between the two and will be kept.' + #13#10#13#10
       + 'Choosing No installs this version and leaves the other one where it '
       + 'is.',
       mbConfirmation, MB_YESNO) <> IDYES then
    Exit;

  RemoveTheOtherCopy(ExtractFileDir(Uninstaller));

  { Said rather than assumed. A folder can be locked by something this has no
    control over, and carrying on quietly would leave the exact situation this
    exists to prevent with nobody told it is still there. }
  if DirExists(ExtractFileDir(Uninstaller)) then
    Result := 'The other copy of Wixen Mail could not be removed completely. '
            + 'Delete this folder by hand, then run setup again:' + #13#10#13#10
            + ExtractFileDir(Uninstaller);
end;

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
;
; skipifdoesntexist, because an uninstall must not depend on the program it is
; removing still being there. Without it, a copy whose executable had already
; gone stopped here with an error, and stopping here means the folder, the
; shortcut and unins000.exe all stay behind: neither installed nor removed, and
; still the first thing the Start Menu offers.
Filename: "{app}\wixen-mail.exe"; Parameters: "--erase-all-data"; RunOnceId: "EraseData"; Flags: runhidden waituntilterminated runascurrentuser skipifdoesntexist

[UninstallDelete]
; Belt and braces for the default location, for the case where the step above
; could not run at all.
Type: filesandordirs; Name: "{localappdata}\wixen-mail"
