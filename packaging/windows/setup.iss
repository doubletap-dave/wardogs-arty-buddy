#ifndef AppVersion
  #define AppVersion "0.0.0"
#endif

[Setup]
AppId={{8F3A1C2E-6B47-4D1A-9E55-2C7A9B0D4E18}
AppName=Wardogs Arty Buddy
AppVersion={#AppVersion}
AppPublisher=Ghostweasel Labs
DefaultDirName={localappdata}\Ghostweasel Labs\Wardogs Arty Buddy
DisableProgramGroupPage=yes
PrivilegesRequired=lowest
OutputDir=..\..\dist
OutputBaseFilename=WardogsArtyBuddy-Setup
Compression=lzma2
SolidCompression=yes
UninstallDisplayName=Wardogs Arty Buddy

[Files]
Source: "..\..\dist\wardogs-arty-buddy-windows-x86_64.exe"; DestDir: "{app}"; DestName: "wardogs-arty-buddy.exe"
Source: "..\..\src\lut\*.bin"; DestDir: "{app}\lut"

[Icons]
Name: "{autoprograms}\Wardogs Arty Buddy"; Filename: "{app}\wardogs-arty-buddy.exe"
