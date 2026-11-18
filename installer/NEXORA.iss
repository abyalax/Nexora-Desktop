#ifndef AppVersion
  #define AppVersion "0.4.0"
#endif
#ifndef SourceDir
  #define SourceDir "..\target\release"
#endif

#define AppName "NEXORA"
#define AppExe "nexora.exe"

[Setup]
AppId={{939A19A7-DB24-45FB-836C-6ED85D256BC3}
AppName={#AppName}
AppVersion={#AppVersion}
AppPublisher=NEXORA contributors
AppPublisherURL=https://github.com/abyalax/Nexora-Desktop
AppSupportURL=https://github.com/abyalax/Nexora-Desktop/issues
DefaultDirName={localappdata}\Programs\{#AppName}
DefaultGroupName={#AppName}
DisableProgramGroupPage=yes
PrivilegesRequired=lowest
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
MinVersion=10.0.22000
LicenseFile=..\LICENSE.md
OutputDir=..\dist
OutputBaseFilename=NEXORA-Setup-{#AppVersion}
Compression=lzma2
SolidCompression=yes
WizardStyle=modern
UninstallDisplayName={#AppName}
UninstallDisplayIcon={app}\{#AppExe}
SetupIconFile=..\assets\nexora.ico
CloseApplications=no

[Tasks]
Name: "autostart"; Description: "Launch NEXORA when I sign in to Windows"; Flags: unchecked
Name: "startmenu"; Description: "Create a Start menu shortcut"

[Files]
Source: "{#SourceDir}\nexora.exe"; DestDir: "{app}"; Flags: ignoreversion
Source: "{#SourceDir}\nexora-guard.exe"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\assets\fonts\*"; DestDir: "{app}\fonts"; Flags: ignoreversion
Source: "..\README.md"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\LICENSE.md"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\docs\PANDUAN.md"; DestDir: "{app}"; Flags: ignoreversion

[Icons]
Name: "{autoprograms}\{#AppName}"; Filename: "{app}\{#AppExe}"; Tasks: startmenu

[Registry]
Root: HKCU; Subkey: "Software\Microsoft\Windows\CurrentVersion\Run"; ValueType: string; ValueName: "NEXORA"; ValueData: """{app}\{#AppExe}"""; Flags: uninsdeletevalue; Tasks: autostart

[Run]
Filename: "{app}\{#AppExe}"; Description: "Launch NEXORA now"; Flags: nowait postinstall skipifsilent

[Code]
procedure StopNexora();
var
  rc: Integer;
begin
  Exec(ExpandConstant('{sys}\taskkill.exe'), '/F /IM nexora.exe', '', SW_HIDE, ewWaitUntilTerminated, rc);
  Sleep(2500);
  Exec(ExpandConstant('{sys}\taskkill.exe'), '/F /IM nexora-guard.exe', '', SW_HIDE, ewWaitUntilTerminated, rc);
  Sleep(300);
end;

function PrepareToInstall(var NeedsRestart: Boolean): String;
begin
  StopNexora();
  Result := '';
end;

function InitializeUninstall(): Boolean;
begin
  StopNexora();
  Result := True;
end;

procedure CurUninstallStepChanged(CurUninstallStep: TUninstallStep);
begin
  if CurUninstallStep = usPostUninstall then
  begin
    if MsgBox('Also delete your NEXORA settings (%APPDATA%\NEXORA)?', mbConfirmation, MB_YESNO or MB_DEFBUTTON2) = IDYES then
      DelTree(ExpandConstant('{userappdata}\NEXORA'), True, True, True);
  end;
end;
