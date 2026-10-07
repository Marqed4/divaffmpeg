; Build: iscc installer\divaffmpeg.iss   (after `cargo build --release`)
; Needs ffmpeg\bin\ffmpeg.exe and ffprobe.exe in the repo root (see README Setup).
#define AppName "DivaFFMPEG"
#define AppVersion "0.1.0"

[Setup]
AppId={{B6C1F0A2-5D0E-4C7B-9A53-D17A0F3E8C21}
AppName={#AppName}
AppVersion={#AppVersion}
AppPublisher=Zachery Francis
; 64-bit binary, so Program Files (not the x86 folder, which is for 32-bit apps)
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
DefaultDirName={autopf}\{#AppName}
DefaultGroupName={#AppName}
PrivilegesRequiredOverridesAllowed=dialog
LicenseFile=..\LICENSE
OutputDir=..\dist
OutputBaseFilename=DivaFFMPEG-Setup-{#AppVersion}
Compression=lzma2/max
SolidCompression=yes
UninstallDisplayIcon={app}\DivaFFMPEG.exe
ChangesEnvironment=yes

[Tasks]
Name: "desktopicon"; Description: "Create a &desktop shortcut"; Flags: unchecked
Name: "addtopath"; Description: "Add to &PATH so you can type 'divaffmpeg diva' in any terminal"

[Files]
Source: "..\target\release\DivaFFMPEG.exe"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\ffmpeg\bin\ffmpeg.exe"; DestDir: "{app}\ffmpeg\bin"; Flags: ignoreversion
Source: "..\ffmpeg\bin\ffprobe.exe"; DestDir: "{app}\ffmpeg\bin"; Flags: ignoreversion
Source: "..\ffmpeg\LICENSE"; DestDir: "{app}\ffmpeg"; Flags: ignoreversion
Source: "..\LICENSE"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\THIRD_PARTY_NOTICES.md"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\README.md"; DestDir: "{app}"; Flags: ignoreversion

[Icons]
Name: "{group}\DivaFFMPEG"; Filename: "{app}\DivaFFMPEG.exe"; Parameters: "diva"
Name: "{group}\Uninstall DivaFFMPEG"; Filename: "{uninstallexe}"
Name: "{autodesktop}\DivaFFMPEG"; Filename: "{app}\DivaFFMPEG.exe"; Parameters: "diva"; Tasks: desktopicon

[Registry]
; Admin install edits system PATH, per-user install edits user PATH.
Root: HKLM; Subkey: "SYSTEM\CurrentControlSet\Control\Session Manager\Environment"; ValueType: expandsz; ValueName: "Path"; ValueData: "{olddata};{app}"; Check: NeedsPath(True); Tasks: addtopath; Flags: preservestringtype
Root: HKCU; Subkey: "Environment"; ValueType: expandsz; ValueName: "Path"; ValueData: "{olddata};{app}"; Check: NeedsPath(False); Tasks: addtopath; Flags: preservestringtype

[Run]
Filename: "{app}\DivaFFMPEG.exe"; Parameters: "diva"; Description: "Launch DivaFFMPEG"; Flags: postinstall nowait skipifsilent shellexec

[Code]
function NeedsPath(Admin: Boolean): Boolean;
var
  Paths, Key: string;
  Root: Integer;
begin
  if Admin <> IsAdminInstallMode then begin
    Result := False;
    exit;
  end;
  if Admin then begin
    Root := HKLM;
    Key := 'SYSTEM\CurrentControlSet\Control\Session Manager\Environment';
  end else begin
    Root := HKCU;
    Key := 'Environment';
  end;
  if not RegQueryStringValue(Root, Key, 'Path', Paths) then
    Paths := '';
  Result := Pos(';' + Uppercase(ExpandConstant('{app}')) + ';', ';' + Uppercase(Paths) + ';') = 0;
end;

procedure CurUninstallStepChanged(CurUninstallStep: TUninstallStep);
var
  Paths, Key, App: string;
  Root, P: Integer;
begin
  if CurUninstallStep <> usPostUninstall then exit;
  if IsAdminInstallMode then begin
    Root := HKLM;
    Key := 'SYSTEM\CurrentControlSet\Control\Session Manager\Environment';
  end else begin
    Root := HKCU;
    Key := 'Environment';
  end;
  if not RegQueryStringValue(Root, Key, 'Path', Paths) then exit;
  App := ExpandConstant('{app}');
  P := Pos(';' + Uppercase(App), Uppercase(Paths));
  if P > 0 then begin
    Delete(Paths, P, Length(App) + 1);
    RegWriteExpandStringValue(Root, Key, 'Path', Paths);
  end;
end;
