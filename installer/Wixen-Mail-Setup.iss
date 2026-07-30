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
// There are two checks below because they find different things and because,
// on the one occasion this has run for real, only one of them worked. Setup
// log #005 shows the prompt appearing with the right folder, so something
// found it; the registry key was still there afterwards, so the registry half
// did not do its part. Which of the two found it is not recorded, and rather
// than pick the flattering explanation, both are kept.
//
// They are worth keeping anyway. The registry check is the only one that can
// find a copy installed somewhere other than the default folder, and the
// filesystem check is the only one that can find a copy Windows has already
// forgotten.

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
// Take the stale entry out of Apps and Features, and say whether that worked.
//
// Not RegDeleteKeyIncludingSubkeys when installing for everybody, which is the
// obvious way and quietly did nothing. The first version of this removed the
// folder and the Start Menu entry and left Apps and Features still offering to
// uninstall a program that no longer existed, on a machine where the key was
// plainly there and the same account could delete it by hand.
//
// Why it failed is not established. The theory at the time was that an elevated
// setup sees a different HKEY_CURRENT_USER, but that does not fit: the folder
// was found through {localappdata} and removed, so setup had the right profile,
// and elevating an account that is already an administrator keeps its hive.
// Writing the guess down as the cause would be worse than saying this much.
//
// reg.exe as the original user is not a workaround for a known fault so much as
// a way of not depending on the answer: whatever setup is running as, it reaches
// the hive of the person who started it, and its exit code says whether it
// worked.
//
// reg.exe run as the original user reaches the right hive whatever setup is
// running as.
function ForgetTheOtherCopy(): Boolean;
var
  Outcome: Integer;
begin
  if not IsAdminInstallMode() then
  begin
    Result := RegDeleteKeyIncludingSubkeys(HKEY_LOCAL_MACHINE, UninstallEntry);
    Exit;
  end;

  Result := ExecAsOriginalUser(
    ExpandConstant('{sys}\reg.exe'),
    'delete "HKCU\' + UninstallEntry + '" /f',
    '', SW_HIDE, ewWaitUntilTerminated, Outcome) and (Outcome = 0);
end;

{ Whether Apps and Features still lists a copy that is not on the disk.

  The state a half-finished cleanup leaves, and one nothing else here would
  find: the folder check has nothing to find, and the registry check cannot see
  the right hive while elevated. Asked through reg.exe as the original user for
  the same reason the delete is. }
function StaleListingExists(): Boolean;
var
  Outcome: Integer;
begin
  if not IsAdminInstallMode() then
  begin
    Result := RegKeyExists(HKEY_LOCAL_MACHINE, UninstallEntry);
    Exit;
  end;

  Result := ExecAsOriginalUser(
    ExpandConstant('{sys}\reg.exe'),
    'query "HKCU\' + UninstallEntry + '"',
    '', SW_HIDE, ewWaitUntilTerminated, Outcome) and (Outcome = 0);
end;

{ Remove the other copy, and return what could not be removed, or ''.

  Every step is reported rather than attempted and forgotten. Each one can fail
  on its own: a folder can be locked, a registry key can be in a hive setup
  cannot see. Silence here is what left an entry in Apps and Features for a
  program that was no longer on the disk, with nobody told. }
function RemoveTheOtherCopy(Folder: String): String;
var
  Shortcut: String;
begin
  Result := '';

  DelTree(Folder, True, True, True);
  if DirExists(Folder) then
    Result := 'Its folder is still there: ' + Folder;

  { Both possible homes for the stale shortcut. Whichever this copy is not
    using is the one that had it. }
  if IsAdminInstallMode() then
    Shortcut := ExpandConstant('{userprograms}\{#AppName}')
  else
    Shortcut := ExpandConstant('{commonprograms}\{#AppName}');
  DelTree(Shortcut, True, True, True);
  if DirExists(Shortcut) then
    Result := Result + #13#10 + 'Its Start Menu entry is still there: ' + Shortcut;

  if not ForgetTheOtherCopy() then
    Result := Result + #13#10
            + 'It is still listed in Apps and Features. Removing it from there '
            + 'will report an error, which is harmless: the program it points '
            + 'at has gone.';
end;

function PrepareToInstall(var NeedsRestart: Boolean): String;
var
  Uninstaller: String;
  Leftovers: String;
begin
  Result := '';

  Uninstaller := UninstallerInTheOtherScope();
  if Uninstaller = '' then
    Uninstaller := StrandedUninstaller();

  if Uninstaller = '' then
  begin
    { Nothing on the disk, but Windows may still be offering to uninstall it.
      Taken away without asking: there is no program left to remove and nothing
      belonging to anybody is touched, so a question here would be a question
      with only one sensible answer. }
    if StaleListingExists() then
      ForgetTheOtherCopy();
    Exit;
  end;

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

  Leftovers := RemoveTheOtherCopy(ExtractFileDir(Uninstaller));

  { Said rather than assumed, and said without stopping the install. This
    version is the one that works; refusing to install it because an old
    shortcut would not delete would leave somebody with only the old one. }
  if Leftovers <> '' then
    MsgBox('Wixen Mail ' + '{#AppVersion}' + ' is installed, but part of the '
           + 'other copy could not be removed:' + #13#10 + Leftovers,
           mbInformation, MB_OK);
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
