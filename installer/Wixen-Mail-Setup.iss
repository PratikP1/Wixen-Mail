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

; Off unless somebody turns it on, because it decides who can read their mail.
; The description says the whole of it rather than a friendly summary, and
; NextButtonClick below asks again before setup goes ahead.
;
; Only offered when installing for everybody. The handler is a COM server the
; Windows indexer loads into its own process, so it has to be registered under
; HKEY_LOCAL_MACHINE, and an install for one user cannot write there. Offering a
; checkbox that would fail is worse than not offering it.
;
; Whether it works at all is not established. Nothing has yet watched the Windows
; indexer ask this handler for a single message. See docs\windows-search.md.
Name: "searchindex"; Description: "Let Windows Search find my mail (experimental, and not yet proven to work). This copies subjects and message text into the Windows Search index, which is not encrypted: any software on this computer can read it."; GroupDescription: "Windows Search:"; Flags: unchecked; Check: IsAdminInstallMode

[Files]
Source: "..\target\release\wixen-mail.exe"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\LICENSE"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\README.md"; DestDir: "{app}"; Flags: ignoreversion
; The guides somebody needs while the application is not working: setting up a
; provider, the keyboard shortcuts, and what to try when mail will not arrive.
; Not recursive, because the folders below docs are notes to ourselves.
Source: "..\docs\*.md"; DestDir: "{app}\docs"; Flags: ignoreversion

; The Windows Search handler and the tool that switches it on and off.
;
; Installed whatever the task above says, and on purpose. Turning this off later
; needs the tool, so leaving a box unticked must not take away the only way to
; undo what ticking it did. They are also the two files somebody needs to try
; this by hand on a machine where the box was never ticked.
;
; Built by scripts/build-installer.sh, which builds the search handler crate as
; a separate step because it has its own target folder. If these are missing,
; this file fails to compile rather than shipping a setup with a checkbox that
; does nothing.
Source: "..\search-handler\target\release\wixen_mail_search.dll"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\search-handler\target\release\wixen-mail-search-setup.exe"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\search-handler\README.md"; DestDir: "{app}\docs"; DestName: "windows-search.md"; Flags: ignoreversion

[Icons]
Name: "{group}\{#AppName}"; Filename: "{app}\wixen-mail.exe"
Name: "{autodesktop}\{#AppName}"; Filename: "{app}\wixen-mail.exe"; Tasks: desktopicon

[Run]
; Two commands rather than one, because the two halves want different accounts.
;
; The registry entries go under HKEY_LOCAL_MACHINE, so that half runs as setup,
; which is elevated.
;
; The crawl scope rule names one person's mail: the URL it registers carries a
; security identifier, and the handler uses it to find whose message store to
; read. So that half runs as whoever started setup. Without runasoriginaluser it
; would name the account typed at the elevation prompt, which on a machine where
; a standard user borrowed an administrator's password is somebody else
; entirely, and the rule would point at a mailbox that does not exist. Nothing
; would report that: the install succeeds and no mail is ever found.
;
; Adding the crawl scope rule did not need administrator rights on the machine
; this was tried on, which is what makes running it as the original user
; possible at all.
Filename: "{app}\wixen-mail-search-setup.exe"; Parameters: "register-classes"; Tasks: searchindex; StatusMsg: "Registering the Windows Search handler..."; Flags: runhidden waituntilterminated
Filename: "{app}\wixen-mail-search-setup.exe"; Parameters: "add-scope"; Tasks: searchindex; StatusMsg: "Telling Windows Search where to look..."; Flags: runhidden waituntilterminated runasoriginaluser

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

// Ask again before letting Windows Search index somebody's mail.
//
// The checkbox already says what happens, and this asks a second time anyway,
// because the two are not the same question. A checkbox in a list is skimmed;
// this stops and requires an answer. What it protects against is somebody
// ticking a row that reads like a feature and finding out afterwards that their
// mail is readable by every program on the machine.
//
// Answering No unticks the box and carries on installing, rather than going
// back to the page. Somebody who has just said "no, do not do that" has given a
// clear answer and should not have to find the checkbox and undo it themselves.
function NextButtonClick(CurPageID: Integer): Boolean;
begin
  Result := True;
  if CurPageID <> wpSelectTasks then
    Exit;
  if not WizardIsTaskSelected('searchindex') then
    Exit;

  if MsgBox(
       'Let Windows Search index your mail?' + #13#10#13#10
       + 'The Windows Search index is not encrypted. It is a database under '
       + 'ProgramData that any software running on this computer can read, and '
       + 'it will keep its own copy of your subjects and message text.' + #13#10#13#10
       + 'Turning this off later stops new mail going in. It does not remove '
       + 'what is already there. Only rebuilding the Windows Search index does '
       + 'that, and it takes hours.' + #13#10#13#10
       + 'This is experimental. Nobody has yet seen the Windows indexer read a '
       + 'single message through it, so it may simply find nothing.' + #13#10#13#10
       + 'Yes turns it on. No installs Wixen Mail without it.',
       mbConfirmation, MB_YESNO) <> IDYES then
    WizardSelectTasks('!searchindex');
end;

// Say so when the step that erases the mail is going to be skipped.
//
// The [UninstallRun] entry below carries skipifdoesntexist, and it has to: an
// uninstall whose executable had already been removed used to stop there and
// leave the folder, the shortcut and unins000.exe behind, which is worse than
// either finished state. What that flag also does is skip the step in silence.
//
// Skipped, the only thing left aiming at the data folder is the
// [UninstallDelete] line at the end of this file, and it knows one location:
// WIXEN_MAIL_DATA moves the folder somewhere that line cannot name. Whether
// {localappdata} there resolves to the person whose mail it is or to the
// account an elevated uninstall is running as has not been measured, and
// guessing at it here would be the mistake ForgetTheOtherCopy above already
// records making. Nothing writes the note in the temporary folder either,
// because the note is written by the program that never ran.
//
// So the mail cache can stay on the disk, unencrypted, with nobody told. An
// uninstall has been seen to end that way, and this is the one branch where no
// part of the machinery would have said a word about it.
//
// Not an attempt to erase it from here. An uninstaller has no way to reach the
// Windows credential store, and it would be guessing at the folder for the
// reason above. Saying plainly what is still there, and where, is what this can
// do honestly.
function TheProgramIsAlreadyGone(): Boolean;
begin
  Result := not FileExists(ExpandConstant('{app}\wixen-mail.exe'));
end;

// Take the Windows Search setup out before the files go.
//
// Here rather than in [UninstallRun] because Inno ignores what a [Run] entry
// exits with, and this is a step that can genuinely fail: Group Policy can lock
// crawl scope changes down. A cleanup that fails silently leaves the indexer
// asking about mail that is no longer on the machine, and nothing anywhere says
// so.
//
// Both commands are run whether or not the checkbox was ever ticked. Each does
// nothing and reports success when there is nothing to undo, and running them
// unconditionally also covers somebody who turned this on by hand afterwards.
procedure CurUninstallStepChanged(CurUninstallStep: TUninstallStep);
var
  Tool: String;
  Outcome: Integer;
  Leftovers: String;
begin
  if CurUninstallStep <> usUninstall then
    Exit;

  { Before the early exit below, because that one is about the search handler
    and this is about somebody's mail. The folder is named as %LOCALAPPDATA%
    rather than expanded here on purpose: expanded, it would print whatever
    account the uninstaller is running as, which is the question above that has
    not been measured. Unexpanded, the person reading it pastes it into File
    Explorer and lands in their own. }
  if TheProgramIsAlreadyGone() then
    MsgBox('Wixen Mail could not clear its own data, because the program was '
           + 'already gone from this computer before the uninstall ran.' + #13#10#13#10
           + 'Your mail and settings are still on this computer, in:' + #13#10
           + '%LOCALAPPDATA%\wixen-mail' + #13#10#13#10
           + 'The downloaded mail in that folder is not encrypted. Paste that '
           + 'path into File Explorer and delete the folder yourself if you '
           + 'want it gone. If you told Wixen Mail to keep its files somewhere '
           + 'else, look there instead.' + #13#10#13#10
           + 'Your saved passwords and sign-in tokens are still in the Windows '
           + 'credential store. Open Credential Manager, choose Windows '
           + 'Credentials, and remove the entries whose names begin with '
           + 'wixen-mail.',
           mbInformation, MB_OK);

  Tool := ExpandConstant('{app}\wixen-mail-search-setup.exe');
  if not FileExists(Tool) then
    Exit;

  Leftovers := '';

  { As the person uninstalling, because the rule names their mail and nobody
    else's. Run elevated this would look for the administrator's rule, find
    none, and report success having removed nothing. }
  if not (ExecAsOriginalUser(Tool, 'remove-scope', '', SW_HIDE,
                             ewWaitUntilTerminated, Outcome) and (Outcome = 0)) then
    Leftovers := 'Windows Search has not been told to stop looking at this mail.';

  { As setup, which is elevated, because the entries are under
    HKEY_LOCAL_MACHINE. }
  if not (Exec(Tool, 'unregister-classes', '', SW_HIDE,
               ewWaitUntilTerminated, Outcome) and (Outcome = 0)) then
    Leftovers := Leftovers + #13#10
               + 'The Windows Search handler is still registered on this computer.';

  if Leftovers <> '' then
    MsgBox('Wixen Mail has been removed, but part of its Windows Search setup '
           + 'could not be:' + #13#10#13#10 + Leftovers + #13#10#13#10
           + 'To finish by hand, open Indexing Options, choose Modify, and '
           + 'untick the Wixen Mail location. Press the Windows key, then type '
           + 'Indexing Options.' + #13#10#13#10
           + 'Anything already in the Windows Search index stays there either '
           + 'way, until the index is rebuilt.',
           mbInformation, MB_OK);
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
