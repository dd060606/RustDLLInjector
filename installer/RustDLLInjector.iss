; Inno Setup script for RustDLLInjector.
; Build the release binary first (./build.ps1) then compile this script:
;   iscc installer\RustDLLInjector.iss
; Output setup goes to installer\dist\.

#define AppName        "RustDLLInjector"
#define AppVersion     "0.1.0"
#define AppExe         "RustDLLInjector.exe"
#define AppSourceRoot  "..\target\release"

[Setup]
AppId={{7A6B5C1E-3F2D-4C9B-9E4A-2C1D8E5F0B23}
AppName={#AppName}
AppVersion={#AppVersion}
AppPublisher={#AppName}
AppSupportURL=https://github.com/dd060606/RustDLLInjector
DefaultDirName={autopf}\{#AppName}
DefaultGroupName={#AppName}
UninstallDisplayIcon={app}\{#AppExe}
DisableProgramGroupPage=yes
OutputDir=dist
OutputBaseFilename={#AppName}-setup-{#AppVersion}
Compression=lzma2/max
SolidCompression=yes
WizardStyle=modern
ArchitecturesInstallIn64BitMode=x64compatible
PrivilegesRequired=lowest
PrivilegesRequiredOverridesAllowed=dialog

[Languages]
Name: "english"; MessagesFile: "compiler:Default.isl"

[Tasks]
Name: "desktopicon"; Description: "Create a &desktop shortcut"; GroupDescription: "Additional shortcuts:"; Flags: unchecked

[Files]
Source: "{#AppSourceRoot}\{#AppExe}"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\README.md"; DestDir: "{app}"; Flags: ignoreversion

[Icons]
Name: "{group}\{#AppName}"; Filename: "{app}\{#AppExe}"
Name: "{group}\Uninstall {#AppName}"; Filename: "{uninstallexe}"
Name: "{autodesktop}\{#AppName}"; Filename: "{app}\{#AppExe}"; Tasks: desktopicon

[Run]
Filename: "{app}\{#AppExe}"; Description: "Launch {#AppName}"; Flags: postinstall nowait skipifsilent
