; Inno Setup script for RustDLLInjector.
; Build both architectures first (./build.ps1 or ./build.sh) then compile
; this script:
;   iscc installer\RustDLLInjector.iss
; Output setup goes to installer\dist\.

#define AppName         "RustDLLInjector"
#define AppVersion      "0.1.0"
#define GuiExe           "RustDLLInjector.exe"
#define CliExe           "RustDLLInjector-CLI.exe"
#define SourceRootX64   "..\target\x86_64-pc-windows-msvc\release"
#define SourceRootX86   "..\target\i686-pc-windows-msvc\release"

[Setup]
AppId={{7A6B5C1E-3F2D-4C9B-9E4A-2C1D8E5F0B23}
AppName={#AppName}
AppVersion={#AppVersion}
AppPublisher={#AppName}
AppSupportURL=https://github.com/dd060606/RustDLLInjector
DefaultDirName={autopf}\{#AppName}
DefaultGroupName={#AppName}
UninstallDisplayIcon={app}\x64\{#GuiExe}
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
Name: "desktopicon_x64"; Description: "Create a &desktop shortcut for the x64 build"; GroupDescription: "Additional shortcuts:"; Flags: unchecked
Name: "desktopicon_x86"; Description: "Create a desktop shortcut for the x&86 build"; GroupDescription: "Additional shortcuts:"; Flags: unchecked

[Files]
Source: "{#SourceRootX64}\{#GuiExe}"; DestDir: "{app}\x64"; Flags: ignoreversion
Source: "{#SourceRootX64}\{#CliExe}"; DestDir: "{app}\x64"; Flags: ignoreversion
Source: "{#SourceRootX86}\{#GuiExe}"; DestDir: "{app}\x86"; Flags: ignoreversion
Source: "{#SourceRootX86}\{#CliExe}"; DestDir: "{app}\x86"; Flags: ignoreversion
Source: "..\README.md"; DestDir: "{app}"; Flags: ignoreversion

[Icons]
Name: "{group}\{#AppName} (x64)"; Filename: "{app}\x64\{#GuiExe}"
Name: "{group}\{#AppName} (x86)"; Filename: "{app}\x86\{#GuiExe}"
Name: "{group}\Uninstall {#AppName}"; Filename: "{uninstallexe}"
Name: "{autodesktop}\{#AppName} (x64)"; Filename: "{app}\x64\{#GuiExe}"; Tasks: desktopicon_x64
Name: "{autodesktop}\{#AppName} (x86)"; Filename: "{app}\x86\{#GuiExe}"; Tasks: desktopicon_x86

[Run]
Filename: "{app}\x64\{#GuiExe}"; Description: "Launch {#AppName} (x64)"; Flags: postinstall nowait skipifsilent unchecked
Filename: "{app}\x86\{#GuiExe}"; Description: "Launch {#AppName} (x86)"; Flags: postinstall nowait skipifsilent unchecked
