#!/usr/bin/env node
// Clean-room, read-only entry point for the M0 Aurora/NWN proof.
//
// This intentionally starts with only `doctor` and `plan`. The live Toolset
// actions must be added as independently tested, targeted adapters; a broad
// `run` command or any global-input fallback would violate the proof contract.

import { createHash } from "node:crypto";
import { existsSync, mkdirSync, readFileSync, statSync, writeFileSync } from "node:fs";
import { basename, dirname, resolve, sep } from "node:path";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const PROJECT_ROOT = resolve(fileURLToPath(new URL("..", import.meta.url)));
const USER_NWN_ROOT = "C:\\Users\\enonw\\Documents\\Neverwinter Nights";
const GAME_BIN = "C:\\Program Files (x86)\\Steam\\steamapps\\common\\Neverwinter Nights\\bin\\win32";

export const M0_PROOF = Object.freeze({
  module: {
    resref: "m2a_m0_proof",
    area: "m2a_m0proof_area",
    hakList: ["znd_tortoise", "m2a_m0_proof"],
  },
  fixtures: Object.freeze([
    Object.freeze({ id: "tortoise", resref: "m2a_m0_tort", appearanceRow: 15108, position: [12, 10] }),
    Object.freeze({ id: "meshy-m0", resref: "m2a_m0p01", appearanceRow: 15109, position: [16, 10] }),
  ]),
  hashes: Object.freeze({
    referenceHak: "3f85946f1e86e20b4d9c6fae311f214587da4009080b64376711f630b0327021",
    m0Hak: "7211c1a016c2b36320f7ce2a399d2833b8f2d7901d2896261d01a93161600200",
    m0Module: "d63b9b71c421ee07ad678619be92b3f624febfecdcd4b09ebd9fdcaa2e13c753",
  }),
  proofDisplay: "\\\\.\\DISPLAY1",
});

export function parseCliArguments(argv) {
  let json = false;
  let command = null;
  let live = false;
  let out = null;
  let reuseExistingSession = false;
  for (let index = 0; index < argv.length; index += 1) {
    const token = argv[index];
    if (token === "--json") {
      if (json) throw new Error("--json may be supplied once");
      json = true;
      continue;
    }
    if (token === "--live") {
      if (live) throw new Error("--live may be supplied once");
      live = true;
      continue;
    }
    if (token === "--reuse-existing-session") {
      if (reuseExistingSession) throw new Error("--reuse-existing-session may be supplied once");
      reuseExistingSession = true;
      continue;
    }
    if (token === "--out") {
      if (out != null) throw new Error("--out may be supplied once");
      const value = argv[index + 1];
      if (value == null || value.startsWith("--")) throw new Error("--out requires a path");
      out = value;
      index += 1;
      continue;
    }
    if (token === "--help" || token === "-h") {
      if (command != null) throw new Error("--help cannot be combined with a command");
      command = "help";
      continue;
    }
    if (token.startsWith("-")) throw new Error(`unknown option: ${token}`);
    if (command != null) throw new Error(`unsupported argument: ${token}`);
    command = token;
  }
  const resolvedCommand = command ?? "help";
  if (!["help", "doctor", "plan", "open", "inspect-open-dialog", "complete-open-dialog"].includes(resolvedCommand)) {
    throw new Error(`unsupported command: ${resolvedCommand}`);
  }
  if (resolvedCommand === "inspect-open-dialog") {
    if (live || reuseExistingSession) throw new Error("--live and --reuse-existing-session are not valid for inspect-open-dialog");
    if (out == null) throw new Error("--out is required for inspect-open-dialog evidence");
  } else if (resolvedCommand === "complete-open-dialog") {
    if (live !== true || reuseExistingSession) throw new Error("--live is required and --reuse-existing-session is not valid for complete-open-dialog");
    if (out == null) throw new Error("--out is required for complete-open-dialog evidence");
  } else if (resolvedCommand !== "open" && (live || out != null || reuseExistingSession)) {
    throw new Error("--live, --out, and --reuse-existing-session are supported only by a live adapter or inspect-open-dialog");
  }
  return {
    json,
    command: resolvedCommand,
    ...(["open", "inspect-open-dialog", "complete-open-dialog"].includes(resolvedCommand) ? {
      ...(["open", "complete-open-dialog"].includes(resolvedCommand) ? { live } : {}),
      ...(resolvedCommand === "open" && reuseExistingSession ? { reuseExistingSession } : {}),
      ...(out != null ? { out } : {}),
    } : {}),
  };
}

export function validateOpenOptions({ live, out, reuseExistingSession = false }) {
  if (live !== true) throw new Error("live required: open has no implicit mutation mode");
  return { ...validateEvidencePath(out, "open"), reuseExistingSession };
}

export function validateEvidencePath(out, command) {
  if (typeof out !== "string" || !out.trim()) throw new Error(`--out is required for ${command} evidence`);
  const outputPath = resolve(PROJECT_ROOT, out);
  const canonicalRoot = `${PROJECT_ROOT}${sep}`;
  if (!outputPath.startsWith(canonicalRoot)) {
    throw new Error(`${command} evidence path must be project-local`);
  }
  return { outputPath, command };
}

// This Delphi/VCL build can owner-draw submenu captions: Windows exposes the
// real File submenu and its command IDs, but not its visible labels. The
// fallback is intentionally narrow and only valid after a current-session
// readback has found an enabled second File item with ID 44. Its use is saved
// in the attempt evidence instead of being hidden as a generic shortcut.
export function resolveOpenMenuItem(items) {
  const labelMatches = items.filter((item) => /^Open(?:\.\.\.)?$/i.test(String(item.label ?? "").replaceAll("&", "").trim())
    && item.enabled === true
    && Number.isInteger(item.id)
    && item.id > 0);
  if (labelMatches.length === 1) {
    return { item: labelMatches[0], identity: "current-menu-label" };
  }
  const labelsUnavailable = items.length >= 2 && items.every((item) => String(item.label ?? "").trim() === "");
  const ownerDrawnFallback = items.filter((item) => item.position === 1 && item.id === 44 && item.enabled === true);
  if (labelMatches.length === 0 && labelsUnavailable && ownerDrawnFallback.length === 1) {
    return { item: ownerDrawnFallback[0], identity: "current-file-menu-position-1-id-44-label-unavailable" };
  }
  throw new Error("open_menu_item_not_uniquely_enabled");
}

export function resolveExactModuleListItem(items, moduleResref) {
  const matches = items
    .map((item, index) => ({ item, index }))
    .filter(({ item }) => item === moduleResref);
  if (matches.length !== 1) throw new Error(`expected_one_exact_module_list_item:${matches.length}`);
  return matches[0];
}

export function buildM0ProofPlan() {
  return {
    schemaVersion: 1,
    goal: "Verify one recovered Meshy M0 model in Aurora Toolset and NWN without treating package installation as runtime proof.",
    module: M0_PROOF.module,
    fixtures: M0_PROOF.fixtures,
    stages: [
      {
        id: "preflight",
        mode: "read-only",
        requires: ["installed hashes", "no conflicting Toolset/NWN session", "approved non-primary display"],
      },
      {
        id: "toolset-open",
        mode: "live-targeted-adapter",
        requires: ["one Toolset session", "current menu/control discovery", "exact module name"],
      },
      {
        id: "toolset-viewport",
        mode: "readback-and-capture",
        requiredBeforeNext: true,
        requires: ["m2a_m0proof_area", "tortoise fixture", "M0 fixture", "safe viewport"],
      },
      {
        id: "test-module",
        mode: "live-targeted-adapter",
        requires: ["currently read Build/Test Module command", "enabled menu item"],
      },
      {
        id: "nwn-proof",
        mode: "runtime-capture",
        requiresEngineLoadLog: true,
        requires: ["fresh engine load log", "tortoise assessment", "M0 assessment", "PNG/MP4/readback packet"],
      },
    ],
    constraints: [
      "Do not use global cursor, keyboard, or mouse input.",
      "Do not use direct nwmain +TestNewModule.",
      "Do not change nwtoolset.ini, MRU, or the game installation.",
      "Do not launch NWN unless the Toolset viewport gate passes in the same attempt.",
    ],
  };
}

export function summarizeDoctor({ paths, processes, displays }) {
  const missing = Object.entries(paths)
    .filter(([, value]) => !value.exists)
    .map(([name]) => name);
  const hashMismatches = Object.entries(paths)
    .filter(([, value]) => value.expectedSha256 != null && value.exists && value.sha256 !== value.expectedSha256)
    .map(([name]) => name);
  const proofDisplay = displays.find((display) => display.deviceName === M0_PROOF.proofDisplay) ?? null;
  const proofDisplayReady = proofDisplay != null && proofDisplay.primary === false;
  const conflictingProcesses = {
    toolset: processes.toolset.length,
    nwmain: processes.nwmain.length,
  };
  return {
    schemaVersion: 1,
    command: "doctor",
    ok: missing.length === 0 && hashMismatches.length === 0 && proofDisplayReady,
    missing,
    hashMismatches,
    paths,
    processes,
    proofDisplay: {
      expected: M0_PROOF.proofDisplay,
      actual: proofDisplay,
      ready: proofDisplayReady,
    },
    conflictingProcesses,
    safeNextAction: missing.length || hashMismatches.length || !proofDisplayReady
      ? "Repair the reported preflight issue; do not start Toolset or NWN."
      : "Use the next independently validated Toolset adapter; no live action is available in this read-only CLI version.",
  };
}

function pathRecord(path, expectedSha256 = null) {
  const exists = existsSync(path);
  if (!exists) return { path, exists: false, expectedSha256, byteLength: null, sha256: null };
  const bytes = readFileSync(path);
  return {
    path,
    exists: true,
    expectedSha256,
    byteLength: statSync(path).size,
    sha256: createHash("sha256").update(bytes).digest("hex"),
  };
}

function collectProcessesAndDisplays() {
  const source = String.raw`
$ErrorActionPreference = 'Stop'
Add-Type -AssemblyName System.Windows.Forms
function Processes([string]$name) {
  @(
    Get-Process $name -ErrorAction SilentlyContinue |
      ForEach-Object {
        [pscustomobject]@{
          id = $_.Id
          responding = $_.Responding
          mainWindowTitle = $_.MainWindowTitle
        }
      }
  )
}
[pscustomobject]@{
  toolset = @(Processes 'nwtoolset')
  nwmain = @(Processes 'nwmain')
  displays = @(
    [System.Windows.Forms.Screen]::AllScreens |
      ForEach-Object {
        [pscustomobject]@{
          deviceName = $_.DeviceName
          primary = $_.Primary
          bounds = [pscustomobject]@{
            x = $_.Bounds.X
            y = $_.Bounds.Y
            width = $_.Bounds.Width
            height = $_.Bounds.Height
          }
        }
      }
  )
} | ConvertTo-Json -Depth 5 -Compress
`;
  const output = execFileSync(
    "powershell.exe",
    ["-NoProfile", "-NonInteractive", "-ExecutionPolicy", "Bypass", "-Command", source],
    { encoding: "utf8", windowsHide: true, timeout: 10_000, maxBuffer: 1024 * 1024 },
  ).trim();
  const parsed = JSON.parse(output);
  return {
    processes: {
      toolset: asArray(parsed.toolset),
      nwmain: asArray(parsed.nwmain),
    },
    displays: asArray(parsed.displays),
  };
}

function collectDoctor() {
  const paths = {
    toolset: pathRecord(resolve(GAME_BIN, "nwtoolset.exe")),
    nwmain: pathRecord(resolve(GAME_BIN, "nwmain.exe")),
    referenceHak: pathRecord(resolve(USER_NWN_ROOT, "hak", "znd_tortoise.hak"), M0_PROOF.hashes.referenceHak),
    m0Hak: pathRecord(resolve(USER_NWN_ROOT, "hak", "m2a_m0_proof.hak"), M0_PROOF.hashes.m0Hak),
    m0Module: pathRecord(resolve(USER_NWN_ROOT, "modules", "m2a_m0_proof.mod"), M0_PROOF.hashes.m0Module),
  };
  const state = collectProcessesAndDisplays();
  return summarizeDoctor({ paths, ...state });
}

function ensureDoctorReady(doctor, { reuseExistingSession }) {
  if (!doctor.ok) {
    throw new Error(`doctor_not_ready:missing=${doctor.missing.join(",")}:hashMismatches=${doctor.hashMismatches.join(",")}`);
  }
  if (doctor.processes.nwmain.length !== 0) {
    throw new Error("nwn_session_exists:do not start or reuse Toolset while NWN is running");
  }
  if (reuseExistingSession) {
    if (doctor.processes.toolset.length !== 1 || doctor.processes.toolset[0].responding !== true) {
      throw new Error("reuse_requires_exactly_one_responding_toolset_session");
    }
  } else if (doctor.processes.toolset.length !== 0) {
    throw new Error("live_session_exists:use PID-first recovery instead of opening another session");
  }
}

function runPowerShell(source, environment, timeoutMs) {
  let output;
  try {
    output = execFileSync(
      "powershell.exe",
      ["-NoProfile", "-NonInteractive", "-ExecutionPolicy", "Bypass", "-Command", source],
      {
        encoding: "utf8",
        windowsHide: true,
        timeout: timeoutMs,
        maxBuffer: 2 * 1024 * 1024,
        env: { ...process.env, ...environment },
      },
    ).trim();
  } catch (error) {
    const diagnostic = [error?.stderr, error?.stdout]
      .map((value) => value == null ? "" : String(value).trim())
      .find(Boolean);
    throw new Error(diagnostic || "powershell_adapter_failed");
  }
  return JSON.parse(output);
}

function openM0Module({ reuseExistingSession }) {
  const source = String.raw`
$ErrorActionPreference = 'Stop'
Add-Type -AssemblyName System.Windows.Forms
Add-Type @'
using System;
using System.Text;
using System.Runtime.InteropServices;
public static class M2AOpen {
  public delegate bool EnumProc(IntPtr hwnd, IntPtr lParam);
  [StructLayout(LayoutKind.Sequential)] public struct MENUITEMINFO {
    public uint cbSize, fMask, fType, fState, wID;
    public IntPtr hSubMenu, hbmpChecked, hbmpUnchecked, dwItemData, dwTypeData;
    public uint cch;
    public IntPtr hbmpItem;
  }
  [DllImport("user32.dll")] public static extern bool EnumWindows(EnumProc callback, IntPtr lParam);
  [DllImport("user32.dll")] public static extern bool EnumChildWindows(IntPtr parent, EnumProc callback, IntPtr lParam);
  [DllImport("user32.dll")] public static extern bool IsWindowVisible(IntPtr hwnd);
  [DllImport("user32.dll")] public static extern uint GetWindowThreadProcessId(IntPtr hwnd, out uint processId);
  [DllImport("user32.dll", CharSet=CharSet.Unicode)] public static extern int GetWindowText(IntPtr hwnd, StringBuilder text, int count);
  [DllImport("user32.dll", CharSet=CharSet.Unicode)] public static extern int GetClassName(IntPtr hwnd, StringBuilder text, int count);
  [DllImport("user32.dll")] public static extern int GetDlgCtrlID(IntPtr hwnd);
  [DllImport("user32.dll", CharSet=CharSet.Unicode)] public static extern IntPtr SendMessage(IntPtr hwnd, uint message, IntPtr wParam, string lParam);
  [DllImport("user32.dll", EntryPoint="SendMessageW")] public static extern IntPtr SendMessageNative(IntPtr hwnd, uint message, IntPtr wParam, IntPtr lParam);
  [DllImport("user32.dll")] public static extern bool PostMessage(IntPtr hwnd, uint message, IntPtr wParam, IntPtr lParam);
  [DllImport("user32.dll")] public static extern IntPtr GetMenu(IntPtr hwnd);
  [DllImport("user32.dll")] public static extern int GetMenuItemCount(IntPtr menu);
  [DllImport("user32.dll")] public static extern IntPtr GetSubMenu(IntPtr menu, int position);
  [DllImport("user32.dll", CharSet=CharSet.Unicode)] public static extern int GetMenuString(IntPtr menu, uint item, StringBuilder text, int count, uint flags);
  [DllImport("user32.dll")] public static extern uint GetMenuItemID(IntPtr menu, int position);
  [DllImport("user32.dll")] public static extern uint GetMenuState(IntPtr menu, uint item, uint flags);
  [DllImport("user32.dll", CharSet=CharSet.Unicode)] public static extern bool GetMenuItemInfo(IntPtr menu, uint item, bool byPosition, ref MENUITEMINFO itemInfo);
  [DllImport("user32.dll")] public static extern bool SetWindowPos(IntPtr hwnd, IntPtr after, int x, int y, int cx, int cy, uint flags);
  public static string MenuLabel(IntPtr menu, int position) {
    IntPtr buffer = Marshal.AllocHGlobal(2048);
    try {
      MENUITEMINFO info = new MENUITEMINFO();
      info.cbSize = (uint)Marshal.SizeOf(typeof(MENUITEMINFO));
      info.fMask = 0x00000040; // MIIM_STRING
      info.dwTypeData = buffer;
      info.cch = 1023;
      if (!GetMenuItemInfo(menu, (uint)position, true, ref info)) return "";
      return Marshal.PtrToStringUni(buffer) ?? "";
    } finally { Marshal.FreeHGlobal(buffer); }
  }
}
'@
function Text([IntPtr]$hwnd) { $buffer = New-Object System.Text.StringBuilder 1024; [void][M2AOpen]::GetWindowText($hwnd,$buffer,$buffer.Capacity); $buffer.ToString() }
function Class([IntPtr]$hwnd) { $buffer = New-Object System.Text.StringBuilder 256; [void][M2AOpen]::GetClassName($hwnd,$buffer,$buffer.Capacity); $buffer.ToString() }
function Windows([uint32]$processId) {
  $script:rows = @()
  [void][M2AOpen]::EnumWindows([M2AOpen+EnumProc]{ param($hwnd,$ignored)
    [uint32]$owner = 0; [void][M2AOpen]::GetWindowThreadProcessId($hwnd,[ref]$owner)
    if ($owner -eq $processId -and [M2AOpen]::IsWindowVisible($hwnd)) {
      $script:rows += [pscustomobject]@{ hwnd=$hwnd; className=(Class $hwnd); title=(Text $hwnd) }
    }
    return $true
  },[IntPtr]::Zero)
  @($script:rows)
}
function Children([IntPtr]$parent) {
  $script:rows = @()
  [void][M2AOpen]::EnumChildWindows($parent,[M2AOpen+EnumProc]{ param($hwnd,$ignored)
    if ([M2AOpen]::IsWindowVisible($hwnd)) { $script:rows += [pscustomobject]@{ hwnd=$hwnd; className=(Class $hwnd); title=(Text $hwnd) } }
    return $true
  },[IntPtr]::Zero)
  @($script:rows)
}
function Move-To-ProofDisplay([IntPtr]$hwnd) {
  $screen = [System.Windows.Forms.Screen]::FromHandle($hwnd)
  if ($screen.DeviceName -ne $env:DISPLAY -or $screen.Primary) {
    $target = @([System.Windows.Forms.Screen]::AllScreens | Where-Object { $_.DeviceName -eq $env:DISPLAY -and -not $_.Primary })
    if ($target.Count -ne 1) { throw 'approved_proof_display_missing' }
    $bounds = $target[0].WorkingArea
    $flags = [uint32]0x0015 # SWP_NOSIZE | SWP_NOZORDER | SWP_NOACTIVATE
    if (-not [M2AOpen]::SetWindowPos($hwnd,[IntPtr]::Zero,$bounds.X + 24,$bounds.Y + 24,0,0,$flags)) { throw 'move_to_proof_display_failed' }
    Start-Sleep -Milliseconds 300
    $screen = [System.Windows.Forms.Screen]::FromHandle($hwnd)
  }
  if ($screen.DeviceName -ne $env:DISPLAY -or $screen.Primary) { throw "window_not_on_proof_display:$($screen.DeviceName):$($screen.Primary)" }
  [pscustomobject]@{ deviceName=$screen.DeviceName; primary=$screen.Primary }
}
function Menu-Text([IntPtr]$menu,[int]$position) { [M2AOpen]::MenuLabel($menu,$position).Replace('&','').Trim() }
function Wait-Until([scriptblock]$predicate,[int]$timeoutMs,[string]$code) {
  $deadline = [DateTime]::UtcNow.AddMilliseconds($timeoutMs)
  do { $value = & $predicate; if ($null -ne $value) { return $value }; Start-Sleep -Milliseconds 250 } while ([DateTime]::UtcNow -lt $deadline)
  throw $code
}
if (@(Get-Process nwmain -ErrorAction SilentlyContinue).Count -ne 0) { throw 'nwn_session_exists_before_open' }
if ($env:REUSE -eq 'true') {
  $sessions = @(Get-Process nwtoolset -ErrorAction SilentlyContinue)
  if ($sessions.Count -ne 1 -or -not $sessions[0].Responding) { throw 'reuse_requires_exactly_one_responding_toolset_session' }
  $process = $sessions[0]
} else {
  if (@(Get-Process nwtoolset -ErrorAction SilentlyContinue).Count -ne 0) { throw 'live_session_exists_before_open' }
  # Keep the inherited process context deterministic.  The prior adapter
  # launch inherited the project directory, which is unrelated to Toolset's
  # local runtime DLL set; a fresh session must resolve that set from the
  # executable directory without mutating any NWN setting.
  $process = Start-Process -FilePath $env:TOOLSET -WorkingDirectory (Split-Path -Parent $env:TOOLSET) -PassThru
}
$frame = Wait-Until {
  $frames = @(Windows ([uint32]$process.Id) | Where-Object { $_.className -eq 'TfrmFrame' })
  if ($frames.Count -eq 1) { $frames[0] } elseif ($frames.Count -gt 1) { throw "unexpected_frame_count:$($frames.Count)" } else { $null }
} 60000 'toolset_frame_start_timeout'
$frameMonitor = Move-To-ProofDisplay $frame.hwnd
$menu = [M2AOpen]::GetMenu($frame.hwnd)
if ($menu -eq [IntPtr]::Zero) { throw 'frame_menu_missing' }
$topCount = [M2AOpen]::GetMenuItemCount($menu)
$filePositions = @(); for ($index = 0; $index -lt $topCount; $index++) { if ((Menu-Text $menu $index) -eq 'File') { $filePositions += $index } }
if ($filePositions.Count -ne 1) { throw "expected_one_file_menu:$($filePositions.Count)" }
$fileMenu = [M2AOpen]::GetSubMenu($menu,$filePositions[0]); if ($fileMenu -eq [IntPtr]::Zero) { throw 'file_submenu_missing' }
$null = [M2AOpen]::SendMessageNative($frame.hwnd,0x0117,$fileMenu,[IntPtr]$filePositions[0]) # WM_INITMENUPOPUP; no command selection
Start-Sleep -Milliseconds 100
$fileCount = [M2AOpen]::GetMenuItemCount($fileMenu)
$fileItems = @()
for ($index = 0; $index -lt $fileCount; $index++) {
  $label = Menu-Text $fileMenu $index
  $state = [M2AOpen]::GetMenuState($fileMenu,[uint32]$index,0x0400)
  $fileItems += [pscustomobject]@{ position=$index; id=[int64][M2AOpen]::GetMenuItemID($fileMenu,$index); label=$label; state=[int64]$state; enabled=(($state -band 3) -eq 0) }
}
$captionMatches = @($fileItems | Where-Object { $_.label -match '^Open(\.\.\.)?$' -and $_.enabled -and $_.id -gt 0 })
$allLabelsUnavailable = $fileItems.Count -ge 2 -and @($fileItems | Where-Object { -not [string]::IsNullOrWhiteSpace($_.label) }).Count -eq 0
$ownerDrawnFallback = @($fileItems | Where-Object { $_.position -eq 1 -and $_.id -eq 44 -and $_.enabled })
if ($captionMatches.Count -eq 1) {
  $openItem = $captionMatches[0]
  $openIdentity = 'current-menu-label'
} elseif ($captionMatches.Count -eq 0 -and $allLabelsUnavailable -and $ownerDrawnFallback.Count -eq 1) {
  $openItem = $ownerDrawnFallback[0]
  $openIdentity = 'current-file-menu-position-1-id-44-label-unavailable'
} else { throw 'open_menu_item_not_uniquely_enabled' }
if (-not [M2AOpen]::PostMessage($frame.hwnd,0x0111,[IntPtr]$openItem.id,[IntPtr]::Zero)) { throw 'open_menu_command_post_failed' }
$dialog = Wait-Until {
  $dialogs = @(Windows ([uint32]$process.Id) | Where-Object { $_.className -eq 'TdlgModuleSelect' -and $_.title -eq 'Open' })
  if ($dialogs.Count -eq 1) { $dialogs[0] } elseif ($dialogs.Count -gt 1) { throw "unexpected_open_dialog_count:$($dialogs.Count)" } else { $null }
} 15000 'open_dialog_timeout'
$dialogMonitor = Move-To-ProofDisplay $dialog.hwnd
$dialogChildren = @(Children $dialog.hwnd)
$listBoxes = @($dialogChildren | Where-Object { $_.className -eq 'TListBox' -and $_.title -eq '' })
$buttons = @($dialogChildren | Where-Object { $_.className -eq 'TButton' -and $_.title -eq 'Open' })
if ($listBoxes.Count -ne 1 -or $buttons.Count -ne 1) { throw "expected_one_module_list_and_open_button:$($listBoxes.Count):$($buttons.Count)" }
[pscustomobject]@{
  ok = $true
  command = 'open'
  reusedExistingSession = ($env:REUSE -eq 'true')
  processId = $process.Id
  module = $env:MODULE
  openMenu = [pscustomobject]@{ selected=$openItem; identity=$openIdentity; items=$fileItems }
  frameHwnd = $frame.hwnd.ToInt64()
  dialogHwnd = $dialog.hwnd.ToInt64()
  monitor = [pscustomobject]@{ frame=$frameMonitor; dialog=$dialogMonitor }
  safety = [pscustomobject]@{
    usesGlobalCursor = $false
    usesGlobalKeyboard = $false
    usesGlobalMouse = $false
    changesIniOrMru = $false
    closesToolset = $false
    method = 'current-menu discovery, targeted WM_COMMAND, then exact Open dialog control discovery; module selection is a separate adapter'
  }
} | ConvertTo-Json -Depth 7 -Compress
`;
  return runPowerShell(source, {
    TOOLSET: resolve(GAME_BIN, "nwtoolset.exe"),
    MODULE: "m2a_m0_proof.mod",
    DISPLAY: M0_PROOF.proofDisplay,
    REUSE: reuseExistingSession ? "true" : "false",
  }, 120_000);
}

function inspectOpenDialog() {
  const source = String.raw`
$ErrorActionPreference = 'Stop'
Add-Type @'
using System;
using System.Text;
using System.Runtime.InteropServices;
public static class M2AOpenDialogInspect {
  public delegate bool EnumProc(IntPtr hwnd, IntPtr lParam);
  [DllImport("user32.dll")] public static extern bool EnumWindows(EnumProc callback, IntPtr lParam);
  [DllImport("user32.dll")] public static extern bool EnumChildWindows(IntPtr parent, EnumProc callback, IntPtr lParam);
  [DllImport("user32.dll")] public static extern bool IsWindowVisible(IntPtr hwnd);
  [DllImport("user32.dll")] public static extern uint GetWindowThreadProcessId(IntPtr hwnd, out uint processId);
  [DllImport("user32.dll", CharSet=CharSet.Unicode)] public static extern int GetWindowText(IntPtr hwnd, StringBuilder text, int count);
  [DllImport("user32.dll", CharSet=CharSet.Unicode)] public static extern int GetClassName(IntPtr hwnd, StringBuilder text, int count);
  [DllImport("user32.dll")] public static extern int GetDlgCtrlID(IntPtr hwnd);
  [DllImport("user32.dll", EntryPoint="SendMessageW")] public static extern IntPtr SendMessageInt(IntPtr hwnd, uint message, IntPtr wParam, IntPtr lParam);
  [DllImport("user32.dll", EntryPoint="SendMessageW", CharSet=CharSet.Unicode)] public static extern IntPtr SendMessageText(IntPtr hwnd, uint message, IntPtr wParam, StringBuilder lParam);
}
'@
function Text([IntPtr]$hwnd) { $buffer = New-Object System.Text.StringBuilder 1024; [void][M2AOpenDialogInspect]::GetWindowText($hwnd,$buffer,$buffer.Capacity); $buffer.ToString() }
function Class([IntPtr]$hwnd) { $buffer = New-Object System.Text.StringBuilder 256; [void][M2AOpenDialogInspect]::GetClassName($hwnd,$buffer,$buffer.Capacity); $buffer.ToString() }
$sessions = @(Get-Process nwtoolset -ErrorAction SilentlyContinue)
if ($sessions.Count -ne 1 -or -not $sessions[0].Responding) { throw 'inspect_requires_exactly_one_responding_toolset_session' }
$processId = [uint32]$sessions[0].Id
$script:topWindows = @()
[void][M2AOpenDialogInspect]::EnumWindows([M2AOpenDialogInspect+EnumProc]{ param($hwnd,$ignored)
  [uint32]$owner = 0; [void][M2AOpenDialogInspect]::GetWindowThreadProcessId($hwnd,[ref]$owner)
  if ($owner -eq $processId -and [M2AOpenDialogInspect]::IsWindowVisible($hwnd)) {
    $script:topWindows += [pscustomobject]@{ hwnd=$hwnd.ToInt64(); className=(Class $hwnd); title=(Text $hwnd) }
  }
  return $true
},[IntPtr]::Zero)
$dialogs = @($script:topWindows | Where-Object { $_.className -eq 'TdlgModuleSelect' -and $_.title -eq 'Open' })
if ($dialogs.Count -ne 1) { throw "expected_one_open_dialog:$($dialogs.Count)" }
$dialogHandle = [IntPtr]$dialogs[0].hwnd
$script:children = @()
[void][M2AOpenDialogInspect]::EnumChildWindows($dialogHandle,[M2AOpenDialogInspect+EnumProc]{ param($hwnd,$ignored)
  $script:children += [pscustomobject]@{
    hwnd = $hwnd.ToInt64()
    className = (Class $hwnd)
    title = (Text $hwnd)
    visible = [M2AOpenDialogInspect]::IsWindowVisible($hwnd)
    controlId = [M2AOpenDialogInspect]::GetDlgCtrlID($hwnd)
  }
  return $true
},[IntPtr]::Zero)
$listBoxes = @($script:children | Where-Object { $_.className -eq 'TListBox' -and $_.visible })
if ($listBoxes.Count -ne 1) { throw "expected_one_visible_module_listbox:$($listBoxes.Count)" }
$listBoxHandle = [IntPtr]$listBoxes[0].hwnd
$listCount = [int][M2AOpenDialogInspect]::SendMessageInt($listBoxHandle,0x018B,[IntPtr]::Zero,[IntPtr]::Zero) # LB_GETCOUNT
if ($listCount -lt 0 -or $listCount -gt 5000) { throw "unsafe_module_list_count:$listCount" }
$listItems = @()
for ($index = 0; $index -lt $listCount; $index++) {
  $length = [int][M2AOpenDialogInspect]::SendMessageInt($listBoxHandle,0x018A,[IntPtr]$index,[IntPtr]::Zero) # LB_GETTEXTLEN
  if ($length -lt 0 -or $length -gt 1024) { throw "unsafe_module_list_item_length:$($index):$($length)" }
  $buffer = New-Object System.Text.StringBuilder ($length + 1)
  [void][M2AOpenDialogInspect]::SendMessageText($listBoxHandle,0x0189,[IntPtr]$index,$buffer) # LB_GETTEXT
  $listItems += $buffer.ToString()
}
$selectedIndex = [int][M2AOpenDialogInspect]::SendMessageInt($listBoxHandle,0x0188,[IntPtr]::Zero,[IntPtr]::Zero) # LB_GETCURSEL
[pscustomobject]@{
  ok = $true
  command = 'inspect-open-dialog'
  processId = $processId
  dialog = $dialogs[0]
  topWindows = @($script:topWindows)
  children = @($script:children)
  moduleList = [pscustomobject]@{ hwnd=$listBoxes[0].hwnd; selectedIndex=$selectedIndex; items=$listItems }
  safety = [pscustomobject]@{
    readOnly = $true
    usesGlobalCursor = $false
    usesGlobalKeyboard = $false
    usesGlobalMouse = $false
    sendsWindowMessages = $false
  }
} | ConvertTo-Json -Depth 7 -Compress
`;
  return runPowerShell(source, {}, 20_000);
}

function completeOpenDialog() {
  const source = String.raw`
$ErrorActionPreference = 'Stop'
Add-Type -AssemblyName System.Windows.Forms
Add-Type @'
using System;
using System.Text;
using System.Runtime.InteropServices;
public static class M2ACompleteOpenDialog {
  public delegate bool EnumProc(IntPtr hwnd, IntPtr lParam);
  [StructLayout(LayoutKind.Sequential)] public struct RECT { public int Left, Top, Right, Bottom; }
  [DllImport("user32.dll")] public static extern bool EnumWindows(EnumProc callback, IntPtr lParam);
  [DllImport("user32.dll")] public static extern bool EnumChildWindows(IntPtr parent, EnumProc callback, IntPtr lParam);
  [DllImport("user32.dll")] public static extern bool IsWindowVisible(IntPtr hwnd);
  [DllImport("user32.dll")] public static extern uint GetWindowThreadProcessId(IntPtr hwnd, out uint processId);
  [DllImport("user32.dll", CharSet=CharSet.Unicode)] public static extern int GetWindowText(IntPtr hwnd, StringBuilder text, int count);
  [DllImport("user32.dll", CharSet=CharSet.Unicode)] public static extern int GetClassName(IntPtr hwnd, StringBuilder text, int count);
  [DllImport("user32.dll")] public static extern int GetDlgCtrlID(IntPtr hwnd);
  [DllImport("user32.dll", EntryPoint="SendMessageW")] public static extern IntPtr SendMessageInt(IntPtr hwnd, uint message, IntPtr wParam, IntPtr lParam);
  [DllImport("user32.dll", EntryPoint="SendMessageW", CharSet=CharSet.Unicode)] public static extern IntPtr SendMessageText(IntPtr hwnd, uint message, IntPtr wParam, StringBuilder lParam);
  [DllImport("user32.dll", EntryPoint="SendMessageW")] public static extern IntPtr SendMessageRect(IntPtr hwnd, uint message, IntPtr wParam, ref RECT lParam);
  [DllImport("user32.dll")] public static extern bool GetClientRect(IntPtr hwnd, out RECT rect);
  [DllImport("user32.dll")] public static extern bool PostMessage(IntPtr hwnd, uint message, IntPtr wParam, IntPtr lParam);
}
'@
<#
// Opens the already-loaded, exact M0 Area through its freshly read native
// context menu.  It neither saves nor changes module content.
function viewM0Area() {
  const source = String.raw\`
$ErrorActionPreference = 'Stop'
Add-Type -AssemblyName System.Windows.Forms
Add-Type @'
using System;
using System.Text;
using System.Runtime.InteropServices;
public static class M2AViewArea {
  public delegate bool EnumProc(IntPtr hwnd, IntPtr lParam);
  [StructLayout(LayoutKind.Sequential)] public struct RECT { public int Left, Top, Right, Bottom; }
  [DllImport("user32.dll")] public static extern bool EnumWindows(EnumProc callback, IntPtr lParam);
  [DllImport("user32.dll")] public static extern bool EnumChildWindows(IntPtr parent, EnumProc callback, IntPtr lParam);
  [DllImport("user32.dll")] public static extern bool IsWindowVisible(IntPtr hwnd);
  [DllImport("user32.dll")] public static extern uint GetWindowThreadProcessId(IntPtr hwnd, out uint processId);
  [DllImport("user32.dll", CharSet=CharSet.Unicode)] public static extern int GetWindowText(IntPtr hwnd, StringBuilder text, int count);
  [DllImport("user32.dll", CharSet=CharSet.Unicode)] public static extern int GetClassName(IntPtr hwnd, StringBuilder text, int count);
  [DllImport("user32.dll", EntryPoint="SendMessageW")] public static extern IntPtr SendMessageInt(IntPtr hwnd, uint message, IntPtr wParam, IntPtr lParam);
  [DllImport("user32.dll", EntryPoint="SendMessageW")] public static extern IntPtr SendMessageRect(IntPtr hwnd, uint message, IntPtr wParam, ref RECT lParam);
  [DllImport("user32.dll")] public static extern bool PostMessage(IntPtr hwnd, uint message, IntPtr wParam, IntPtr lParam);
  [DllImport("user32.dll")] public static extern bool GetClientRect(IntPtr hwnd, out RECT rect);
  [DllImport("user32.dll")] public static extern IntPtr GetMenu(IntPtr hwnd);
  [DllImport("user32.dll")] public static extern int GetMenuItemCount(IntPtr menu);
  [DllImport("user32.dll")] public static extern uint GetMenuItemID(IntPtr menu, int position);
  [DllImport("user32.dll")] public static extern uint GetMenuState(IntPtr menu, uint item, uint flags);
  [DllImport("user32.dll", CharSet=CharSet.Unicode)] public static extern int GetMenuString(IntPtr menu, uint item, StringBuilder text, int count, uint flags);
  [DllImport("kernel32.dll")] public static extern IntPtr OpenProcess(uint access, bool inheritHandle, uint processId);
  [DllImport("kernel32.dll")] public static extern bool CloseHandle(IntPtr handle);
  [DllImport("kernel32.dll")] public static extern IntPtr VirtualAllocEx(IntPtr process, IntPtr address, uint size, uint allocationType, uint protection);
  [DllImport("kernel32.dll")] public static extern bool VirtualFreeEx(IntPtr process, IntPtr address, uint size, uint freeType);
  [DllImport("kernel32.dll")] public static extern bool WriteProcessMemory(IntPtr process, IntPtr address, byte[] buffer, uint size, out IntPtr written);
  [DllImport("kernel32.dll")] public static extern bool ReadProcessMemory(IntPtr process, IntPtr address, byte[] buffer, uint size, out IntPtr read);
}
'@
function Text([IntPtr]$hwnd) { $buffer = [Text.StringBuilder]::new(1024); [void][M2AViewArea]::GetWindowText($hwnd, $buffer, $buffer.Capacity); $buffer.ToString() }
function Class([IntPtr]$hwnd) { $buffer = [Text.StringBuilder]::new(256); [void][M2AViewArea]::GetClassName($hwnd, $buffer, $buffer.Capacity); $buffer.ToString() }
function ProcessWindows([uint32]$processId) {
  $script:rows = @()
  [void][M2AViewArea]::EnumWindows([M2AViewArea+EnumProc]{ param($hwnd, $ignored)
    [uint32]$owner = 0; [void][M2AViewArea]::GetWindowThreadProcessId($hwnd, [ref]$owner)
    if ($owner -eq $processId -and [M2AViewArea]::IsWindowVisible($hwnd)) { $script:rows += [pscustomobject]@{ hwnd=$hwnd; className=(Class $hwnd); title=(Text $hwnd) } }
    return $true
  }, [IntPtr]::Zero)
  @($script:rows)
}
function ChildWindows([IntPtr]$parent) {
  $script:rows = @()
  [void][M2AViewArea]::EnumChildWindows($parent, [M2AViewArea+EnumProc]{ param($hwnd, $ignored)
    if ([M2AViewArea]::IsWindowVisible($hwnd)) { $script:rows += [pscustomobject]@{ hwnd=$hwnd; className=(Class $hwnd); title=(Text $hwnd) } }
    return $true
  }, [IntPtr]::Zero)
  @($script:rows)
}
function ReadTreeText([IntPtr]$tree, [IntPtr]$item, [IntPtr]$process) {
  $textAddress = [M2AViewArea]::VirtualAllocEx($process, [IntPtr]::Zero, 1024, 0x3000, 0x04)
  $itemAddress = [M2AViewArea]::VirtualAllocEx($process, [IntPtr]::Zero, 40, 0x3000, 0x04)
  if ($textAddress -eq [IntPtr]::Zero -or $itemAddress -eq [IntPtr]::Zero) { throw 'tree_remote_memory_allocation_failed' }
  try {
    $native = [byte[]]::new(40)
    [BitConverter]::GetBytes([uint32]1).CopyTo($native, 0)
    [BitConverter]::GetBytes([uint32]$item.ToInt64()).CopyTo($native, 4)
    [BitConverter]::GetBytes([uint32]$textAddress.ToInt64()).CopyTo($native, 16)
    [BitConverter]::GetBytes([int]512).CopyTo($native, 20)
    [IntPtr]$written = [IntPtr]::Zero
    if (-not [M2AViewArea]::WriteProcessMemory($process, $itemAddress, $native, [uint32]$native.Length, [ref]$written) -or $written.ToInt64() -ne $native.Length) { throw 'tree_item_write_failed' }
    if ([M2AViewArea]::SendMessageInt($tree, 0x110C, [IntPtr]::Zero, $itemAddress) -eq [IntPtr]::Zero) { throw 'tree_item_text_read_failed' }
    $bytes = [byte[]]::new(1024); [IntPtr]$read = [IntPtr]::Zero
    if (-not [M2AViewArea]::ReadProcessMemory($process, $textAddress, $bytes, [uint32]$bytes.Length, [ref]$read)) { throw 'tree_text_buffer_read_failed' }
    $length = [Array]::IndexOf($bytes, [byte]0); if ($length -lt 0) { $length = $bytes.Length }
    [Text.Encoding]::Default.GetString($bytes, 0, $length)
  } finally {
    if ($itemAddress -ne [IntPtr]::Zero) { [void][M2AViewArea]::VirtualFreeEx($process, $itemAddress, 0, 0x8000) }
    if ($textAddress -ne [IntPtr]::Zero) { [void][M2AViewArea]::VirtualFreeEx($process, $textAddress, 0, 0x8000) }
  }
}
function TreeNext([IntPtr]$tree, [int]$relation, [IntPtr]$item) { [M2AViewArea]::SendMessageInt($tree, 0x110A, [IntPtr]$relation, $item) }
function FindDirectChild([IntPtr]$tree, [IntPtr]$parent, [IntPtr]$process, [string]$expected) {
  $found = @(); $cursor = TreeNext $tree 4 $parent
  while ($cursor -ne [IntPtr]::Zero) { if ((ReadTreeText $tree $cursor $process) -eq $expected) { $found += $cursor }; $cursor = TreeNext $tree 1 $cursor }
  if ($found.Count -ne 1) { throw "expected_one_tree_item:$expected:$($found.Count)" }; $found[0]
}
function PackPoint([int]$x, [int]$y) { [IntPtr][int64](($y -shl 16) -bor ($x -band 0xffff)) }
function WaitUntil([scriptblock]$predicate, [int]$timeoutMs, [string]$code) {
  $deadline = [DateTime]::UtcNow.AddMilliseconds($timeoutMs)
  do { $value = & $predicate; if ($null -ne $value) { return $value }; Start-Sleep -Milliseconds 150 } while ([DateTime]::UtcNow -lt $deadline)
  throw $code
}
if (@(Get-Process nwmain -ErrorAction SilentlyContinue).Count -ne 0) { throw 'nwn_session_exists_before_view_area' }
$sessions = @(Get-Process nwtoolset -ErrorAction SilentlyContinue); if ($sessions.Count -ne 1 -or -not $sessions[0].Responding) { throw "view_area_requires_exactly_one_responding_toolset:$($sessions.Count)" }
$process = $sessions[0]; $windows = @(ProcessWindows ([uint32]$process.Id)); $frames = @($windows | Where-Object { $_.className -eq 'TfrmFrame' -and $_.title.TrimEnd('*').EndsWith(" - $env:MODULE.mod", [StringComparison]::OrdinalIgnoreCase) })
if ($frames.Count -ne 1) { throw "expected_exact_loaded_module_frame:$($frames.Count)" }; $frame = $frames[0]; $screen = [Windows.Forms.Screen]::FromHandle($frame.hwnd)
if ($screen.DeviceName -ne $env:DISPLAY -or $screen.Primary) { throw "module_frame_not_on_proof_display:$($screen.DeviceName):$($screen.Primary)" }
$danger = @($windows | Where-Object { $_.title -match 'Access violation|List index out of bounds|Read of address|Write of address|recover|Loading' }); if ($danger.Count -ne 0) { throw 'toolset_unsafe_before_view_area' }
$trees = @(ChildWindows $frame.hwnd | Where-Object { $_.className -eq 'TTreeView' }); if ($trees.Count -ne 1) { throw "expected_one_visible_tree:$($trees.Count)" }; $tree = $trees[0].hwnd
$processHandle = [M2AViewArea]::OpenProcess(0x0038, $false, [uint32]$process.Id); if ($processHandle -eq [IntPtr]::Zero) { throw 'tree_process_open_failed' }
try {
  $areasRoot = FindDirectChild $tree [IntPtr]::Zero $processHandle 'Areas'
  [void][M2AViewArea]::SendMessageInt($tree, 0x1102, [IntPtr]2, $areasRoot)
  Start-Sleep -Milliseconds 150
  $area = FindDirectChild $tree $areasRoot $processHandle $env:AREA_LABEL
  [void][M2AViewArea]::SendMessageInt($tree, 0x1114, [IntPtr]::Zero, $area)
  [void][M2AViewArea]::SendMessageInt($tree, 0x110B, [IntPtr]9, $area)
  $caret = TreeNext $tree 9 [IntPtr]::Zero
  if ($caret -ne $area -or (ReadTreeText $tree $caret $processHandle) -ne $env:AREA_LABEL) { throw 'area_tree_selection_readback_failed' }
  $rect = New-Object 'M2AViewArea+RECT'; $rect.Left = [int]$area.ToInt64()
  if ([M2AViewArea]::SendMessageRect($tree, 0x1104, [IntPtr]::Zero, [ref]$rect) -eq [IntPtr]::Zero) { throw 'area_tree_item_rect_unavailable' }
  $client = New-Object 'M2AViewArea+RECT'; if (-not [M2AViewArea]::GetClientRect($tree, [ref]$client)) { throw 'area_tree_client_rect_unavailable' }
  $x = [int](($rect.Left + $rect.Right) / 2); $y = [int](($rect.Top + $rect.Bottom) / 2)
  if ($rect.Right -le $rect.Left -or $rect.Bottom -le $rect.Top -or $x -lt $client.Left -or $x -ge $client.Right -or $y -lt $client.Top -or $y -ge $client.Bottom) { throw 'area_tree_item_rect_outside_client' }
  $point = PackPoint $x $y
  if (-not [M2AViewArea]::PostMessage($tree, 0x0204, [IntPtr]2, $point) -or -not [M2AViewArea]::PostMessage($tree, 0x0205, [IntPtr]::Zero, $point)) { throw 'area_tree_context_post_failed' }
  $popup = WaitUntil {
    $script:popups = @(); [void][M2AViewArea]::EnumWindows([M2AViewArea+EnumProc]{ param($hwnd, $ignored) [uint32]$owner = 0; [void][M2AViewArea]::GetWindowThreadProcessId($hwnd, [ref]$owner); if ($owner -eq [uint32]$process.Id -and [M2AViewArea]::IsWindowVisible($hwnd) -and (Class $hwnd) -eq '#32768') { $script:popups += $hwnd }; $true }, [IntPtr]::Zero)
    if ($script:popups.Count -eq 1) { $script:popups[0] } else { $null }
  } 5000 'area_context_popup_missing'
  $menu = [M2AViewArea]::GetMenu($popup); if ($menu -eq [IntPtr]::Zero -or [M2AViewArea]::GetMenuItemCount($menu) -lt 1) { throw 'area_context_menu_missing' }
  $caption = [Text.StringBuilder]::new(256); [void][M2AViewArea]::GetMenuString($menu, 0, $caption, $caption.Capacity, 0); $viewId = [M2AViewArea]::GetMenuItemID($menu, 0); $viewState = [M2AViewArea]::GetMenuState($menu, 0, 0)
  if ($caption.ToString() -ne '&View Area' -or $viewId -ne 4 -or (($viewState -band 3) -ne 0)) { throw "validated_view_area_menu_item_missing:$($caption.ToString()):$viewId:$viewState" }
  if (-not [M2AViewArea]::PostMessage($popup, 0x0102, [IntPtr][int][char]'V', [IntPtr]::Zero)) { throw 'view_area_popup_accelerator_failed' }
  $viewer = WaitUntil {
    $candidates = @(ChildWindows $frame.hwnd | Where-Object { $_.className -eq 'TfrmViewerArea' -and $_.title.Trim().TrimStart('-').Trim() -eq $env:AREA_LABEL })
    if ($candidates.Count -eq 1) { $candidates[0] } else { $null }
  } 30000 'exact_area_viewer_missing'
  $viewports = @(ChildWindows $viewer.hwnd | Where-Object { $_.className -eq 'TScrollBox' }); $viewport = $null
  foreach ($candidate in $viewports) { $candidateRect = New-Object 'M2AViewArea+RECT'; if ([M2AViewArea]::GetClientRect($candidate.hwnd, [ref]$candidateRect) -and ($candidateRect.Right - $candidateRect.Left) -ge 400 -and ($candidateRect.Bottom - $candidateRect.Top) -ge 300) { $viewport = $candidate; break } }
  if ($null -eq $viewport) { throw 'area_viewport_not_ready' }
  $after = @(ProcessWindows ([uint32]$process.Id)); if (@($after | Where-Object { $_.className -eq '#32768' }).Count -ne 0) { throw 'area_context_popup_left_open' }
  [pscustomobject]@{ ok=$true; command='view-area'; processId=$process.Id; module=$env:MODULE; area=[pscustomobject]@{ resref=$env:AREA_RESREF; label=$env:AREA_LABEL; treeHwnd=$tree.ToInt64(); treeItem=$area.ToInt64() }; viewer=[pscustomobject]@{ hwnd=$viewer.hwnd.ToInt64(); title=$viewer.title; viewportHwnd=$viewport.hwnd.ToInt64() }; monitor=[pscustomobject]@{ deviceName=$screen.DeviceName; primary=$screen.Primary }; menu=[pscustomobject]@{ commandId=$viewId; text=$caption.ToString(); accelerator='V' }; safety=[pscustomobject]@{ usesGlobalCursor=$false; usesGlobalKeyboard=$false; usesGlobalMouse=$false; changesIniOrMru=$false; closesToolset=$false; mutatesModule=$false; method='remote TreeView identity readback, targeted context popup, verified View Area accelerator, exact viewer and viewport readback' } } | ConvertTo-Json -Depth 8 -Compress
} finally { [void][M2AViewArea]::CloseHandle($processHandle) }
\`;
  return runPowerShell(source, {
    DISPLAY: M0_PROOF.proofDisplay,
    MODULE: M0_PROOF.module.resref,
    AREA_RESREF: M0_PROOF.module.area,
    AREA_LABEL: M0_PROOF.module.areaLabel,
  }, 60_000);
}
#>
function Text([IntPtr]$hwnd) { $buffer = New-Object System.Text.StringBuilder 1024; [void][M2ACompleteOpenDialog]::GetWindowText($hwnd,$buffer,$buffer.Capacity); $buffer.ToString() }
function Class([IntPtr]$hwnd) { $buffer = New-Object System.Text.StringBuilder 256; [void][M2ACompleteOpenDialog]::GetClassName($hwnd,$buffer,$buffer.Capacity); $buffer.ToString() }
function Process-Windows([uint32]$processId) {
  $script:rows = @()
  [void][M2ACompleteOpenDialog]::EnumWindows([M2ACompleteOpenDialog+EnumProc]{ param($hwnd,$ignored)
    [uint32]$owner = 0; [void][M2ACompleteOpenDialog]::GetWindowThreadProcessId($hwnd,[ref]$owner)
    if ($owner -eq $processId -and [M2ACompleteOpenDialog]::IsWindowVisible($hwnd)) {
      $script:rows += [pscustomobject]@{ hwnd=$hwnd; className=(Class $hwnd); title=(Text $hwnd) }
    }
    return $true
  },[IntPtr]::Zero)
  @($script:rows)
}
function Visible-Children([IntPtr]$parent) {
  $script:rows = @()
  [void][M2ACompleteOpenDialog]::EnumChildWindows($parent,[M2ACompleteOpenDialog+EnumProc]{ param($hwnd,$ignored)
    if ([M2ACompleteOpenDialog]::IsWindowVisible($hwnd)) { $script:rows += [pscustomobject]@{ hwnd=$hwnd; className=(Class $hwnd); title=(Text $hwnd) } }
    return $true
  },[IntPtr]::Zero)
  @($script:rows)
}
function List-Text([IntPtr]$listBox,[int]$index) {
  $length = [int][M2ACompleteOpenDialog]::SendMessageInt($listBox,0x018A,[IntPtr]$index,[IntPtr]::Zero) # LB_GETTEXTLEN
  if ($length -lt 0 -or $length -gt 1024) { throw "unsafe_module_list_item_length:$($index):$($length)" }
  $buffer = New-Object System.Text.StringBuilder ($length + 1)
  [void][M2ACompleteOpenDialog]::SendMessageText($listBox,0x0189,[IntPtr]$index,$buffer) # LB_GETTEXT
  $buffer.ToString()
}
function Wait-Until([scriptblock]$predicate,[int]$timeoutMs,[string]$code) {
  $deadline = [DateTime]::UtcNow.AddMilliseconds($timeoutMs)
  do { $value = & $predicate; if ($null -ne $value) { return $value }; Start-Sleep -Milliseconds 250 } while ([DateTime]::UtcNow -lt $deadline)
  throw $code
}
if (@(Get-Process nwmain -ErrorAction SilentlyContinue).Count -ne 0) { throw 'nwn_session_exists_before_complete_open' }
$sessions = @(Get-Process nwtoolset -ErrorAction SilentlyContinue)
if ($sessions.Count -ne 1 -or -not $sessions[0].Responding) { throw 'complete_open_requires_exactly_one_responding_toolset_session' }
$process = $sessions[0]
$windows = @(Process-Windows ([uint32]$process.Id))
$frames = @($windows | Where-Object { $_.className -eq 'TfrmFrame' })
if ($frames.Count -ne 1) { throw "expected_one_toolset_frame:$($frames.Count)" }
$initialFrameTitle = $frames[0].title
$dialogs = @($windows | Where-Object { $_.className -eq 'TdlgModuleSelect' -and $_.title -eq 'Open' })
if ($dialogs.Count -ne 1) { throw "expected_one_open_dialog:$($dialogs.Count)" }
$dialog = $dialogs[0]
$dialogScreen = [System.Windows.Forms.Screen]::FromHandle($dialog.hwnd)
if ($dialogScreen.DeviceName -ne $env:DISPLAY -or $dialogScreen.Primary) { throw "open_dialog_not_on_proof_display:$($dialogScreen.DeviceName):$($dialogScreen.Primary)" }
$children = @(Visible-Children $dialog.hwnd)
$listBoxes = @($children | Where-Object { $_.className -eq 'TListBox' -and $_.title -eq '' })
$buttons = @($children | Where-Object { $_.className -eq 'TButton' -and $_.title -eq 'Open' })
if ($listBoxes.Count -ne 1 -or $buttons.Count -ne 1) { throw "expected_one_module_list_and_open_button:$($listBoxes.Count):$($buttons.Count)" }
$listBox = $listBoxes[0].hwnd
$count = [int][M2ACompleteOpenDialog]::SendMessageInt($listBox,0x018B,[IntPtr]::Zero,[IntPtr]::Zero) # LB_GETCOUNT
if ($count -lt 0 -or $count -gt 5000) { throw "unsafe_module_list_count:$count" }
$matches = @()
for ($index = 0; $index -lt $count; $index++) { if ((List-Text $listBox $index) -eq $env:MODULE) { $matches += $index } }
if ($matches.Count -ne 1) { throw "expected_one_exact_module_list_item:$($matches.Count)" }
$selectedIndex = [int][M2ACompleteOpenDialog]::SendMessageInt($listBox,0x0186,[IntPtr]$matches[0],[IntPtr]::Zero) # LB_SETCURSEL
if ($selectedIndex -ne $matches[0]) { throw "module_list_selection_failed:$selectedIndex" }
$readbackIndex = [int][M2ACompleteOpenDialog]::SendMessageInt($listBox,0x0188,[IntPtr]::Zero,[IntPtr]::Zero) # LB_GETCURSEL
if ($readbackIndex -ne $matches[0] -or (List-Text $listBox $readbackIndex) -ne $env:MODULE) { throw 'module_list_selection_readback_failed' }
$scrollResult = [int][M2ACompleteOpenDialog]::SendMessageInt($listBox,0x0197,[IntPtr]$readbackIndex,[IntPtr]::Zero) # LB_SETTOPINDEX
if ($scrollResult -lt 0) { throw "module_list_scroll_failed:$scrollResult" }
$itemRect = New-Object M2ACompleteOpenDialog+RECT
$itemRectResult = [int][M2ACompleteOpenDialog]::SendMessageRect($listBox,0x0198,[IntPtr]$readbackIndex,[ref]$itemRect) # LB_GETITEMRECT
$clientRect = New-Object M2ACompleteOpenDialog+RECT
if ($itemRectResult -eq 0 -or -not [M2ACompleteOpenDialog]::GetClientRect($listBox,[ref]$clientRect)) { throw 'module_list_item_rect_unavailable' }
$centerX = [int](($itemRect.Left + $itemRect.Right) / 2)
$centerY = [int](($itemRect.Top + $itemRect.Bottom) / 2)
if ($itemRect.Right -le $itemRect.Left -or $itemRect.Bottom -le $itemRect.Top -or $centerX -lt $clientRect.Left -or $centerX -ge $clientRect.Right -or $centerY -lt $clientRect.Top -or $centerY -ge $clientRect.Bottom) { throw 'module_list_item_rect_outside_client' }
$point = [int64](($centerY -shl 16) -bor ($centerX -band 0xffff))
[void][M2ACompleteOpenDialog]::SendMessageInt($listBox,0x0201,[IntPtr]1,[IntPtr]$point) # target-local WM_LBUTTONDOWN
[void][M2ACompleteOpenDialog]::SendMessageInt($listBox,0x0202,[IntPtr]::Zero,[IntPtr]$point) # target-local WM_LBUTTONUP
Start-Sleep -Milliseconds 100
$clickReadbackIndex = [int][M2ACompleteOpenDialog]::SendMessageInt($listBox,0x0188,[IntPtr]::Zero,[IntPtr]::Zero)
if ($clickReadbackIndex -ne $readbackIndex -or (List-Text $listBox $clickReadbackIndex) -ne $env:MODULE) { throw 'module_list_target_click_readback_failed' }
$controlId = [M2ACompleteOpenDialog]::GetDlgCtrlID($listBox)
if ($controlId -eq 0) { throw 'module_list_control_id_missing' }
$selectionChange = [int64]((1 -shl 16) -bor ($controlId -band 0xffff)) # WM_COMMAND / LBN_SELCHANGE
[void][M2ACompleteOpenDialog]::SendMessageInt($dialog.hwnd,0x0111,[IntPtr]$selectionChange,$listBox)
Start-Sleep -Milliseconds 100
if (-not [M2ACompleteOpenDialog]::PostMessage($buttons[0].hwnd,0x00F5,[IntPtr]::Zero,[IntPtr]::Zero)) { throw 'open_dialog_button_post_failed' }
$title = Wait-Until {
  $currentWindows = @(Process-Windows ([uint32]$process.Id))
  $remainingDialogs = @($currentWindows | Where-Object { $_.className -eq 'TdlgModuleSelect' -and $_.title -eq 'Open' })
  $currentFrames = @($currentWindows | Where-Object { $_.className -eq 'TfrmFrame' })
  if ($remainingDialogs.Count -eq 0 -and $currentFrames.Count -eq 1 -and $currentFrames[0].title -ne $initialFrameTitle) { $currentFrames[0].title } else { $null }
} 60000 'target_module_load_timeout'
$finalWindows = @(Process-Windows ([uint32]$process.Id))
$danger = @($finalWindows | Where-Object { $_.title -match 'Access violation|List index out of bounds|Read of address|Write of address|recover' })
if ($danger.Count -ne 0) { throw 'toolset_error_or_recovery_dialog_detected' }
[pscustomobject]@{
  ok = $true
  command = 'complete-open-dialog'
  processId = $process.Id
  module = $env:MODULE
  initialFrameTitle = $initialFrameTitle
  titleAfter = $title
  selectedListIndex = $clickReadbackIndex
  monitor = [pscustomobject]@{ deviceName=$dialogScreen.DeviceName; primary=$dialogScreen.Primary }
  safety = [pscustomobject]@{
    usesGlobalCursor = $false
    usesGlobalKeyboard = $false
    usesGlobalMouse = $false
    changesIniOrMru = $false
    closesToolset = $false
    method = 'current dialog discovery, exact list-item readback, target-local list mouse messages and LBN_SELCHANGE, then targeted Open BM_CLICK and frame-title readback'
  }
} | ConvertTo-Json -Depth 6 -Compress
`;
  return runPowerShell(source, {
    DISPLAY: M0_PROOF.proofDisplay,
    MODULE: M0_PROOF.module.resref,
  }, 90_000);
}

function writeEvidence(outputPath, result) {
  if (existsSync(outputPath)) throw new Error(`evidence_already_exists:${outputPath}`);
  mkdirSync(dirname(outputPath), { recursive: true });
  writeFileSync(outputPath, `${JSON.stringify(result, null, 2)}\n`, { encoding: "utf8", flag: "wx" });
}

function asArray(value) {
  return value == null ? [] : Array.isArray(value) ? value : [value];
}

function usage() {
  return [
    "m2a-aurora-proof - fail-closed M0 Aurora/NWN proof adapter",
    "",
    "Usage:",
    "  node tools/m2a-aurora-proof.mjs --json doctor",
    "  node tools/m2a-aurora-proof.mjs --json plan",
    "  node tools/m2a-aurora-proof.mjs --json open --live --out proof-output/m0-attempt/open.json",
    "  node tools/m2a-aurora-proof.mjs --json open --live --reuse-existing-session --out proof-output/m0-attempt/open-reuse.json",
    "  node tools/m2a-aurora-proof.mjs --json inspect-open-dialog --out proof-output/m0-attempt/open-dialog.json",
    "  node tools/m2a-aurora-proof.mjs --json complete-open-dialog --live --out proof-output/m0-attempt/module-open.json",
    "",
    "Commands:",
    "  doctor    Read installation hashes, live process state, and proof display state.",
    "  plan      Print the mandatory Toolset -> NWN gate sequence without a live action.",
    "  open      Open exactly m2a_m0_proof.mod by current-menu discovery; requires --live and a new project-local --out file.",
    "  inspect-open-dialog  Read the current Toolset Open dialog controls; requires a new project-local --out file.",
    "  complete-open-dialog  Select the exact M0 module in the current Open dialog and verify the dialog closes.",
    "",
    "Only the independently tested open adapter is live. Viewport and Test Module",
    "remain unavailable until their own control-discovery and readback adapters exist.",
  ].join("\n");
}

function emit(value, json) {
  if (json) {
    process.stdout.write(`${JSON.stringify(value, null, 2)}\n`);
  } else if (typeof value === "string") {
    process.stdout.write(`${value}\n`);
  } else {
    process.stdout.write(`${JSON.stringify(value, null, 2)}\n`);
  }
}

function run() {
  const argumentsValue = parseCliArguments(process.argv.slice(2));
  if (argumentsValue.command === "help") {
    emit(usage(), argumentsValue.json);
    return;
  }
  if (argumentsValue.command === "plan") {
    emit(buildM0ProofPlan(), argumentsValue.json);
    return;
  }
  if (argumentsValue.command === "doctor") {
    emit(collectDoctor(), argumentsValue.json);
    return;
  }
  if (argumentsValue.command === "inspect-open-dialog") {
    const { outputPath } = validateEvidencePath(argumentsValue.out, "inspect-open-dialog");
    if (existsSync(outputPath)) throw new Error(`evidence_already_exists:${outputPath}`);
    const result = { ...inspectOpenDialog(), evidencePath: outputPath };
    writeEvidence(outputPath, result);
    emit(result, argumentsValue.json);
    return;
  }
  if (argumentsValue.command === "complete-open-dialog") {
    const { outputPath } = validateEvidencePath(argumentsValue.out, "complete-open-dialog");
    if (existsSync(outputPath)) throw new Error(`evidence_already_exists:${outputPath}`);
    try {
      const result = { ...completeOpenDialog(), evidencePath: outputPath };
      writeEvidence(outputPath, result);
      emit(result, argumentsValue.json);
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error);
      writeEvidence(outputPath, { ok: false, command: "complete-open-dialog", evidencePath: outputPath, error: message });
      throw error;
    }
    return;
  }
  const { outputPath, reuseExistingSession } = validateOpenOptions(argumentsValue);
  if (existsSync(outputPath)) throw new Error(`evidence_already_exists:${outputPath}`);
  let doctor = null;
  try {
    doctor = collectDoctor();
    ensureDoctorReady(doctor, { reuseExistingSession });
    const opened = openM0Module({ reuseExistingSession });
    const result = { ...opened, doctor, evidencePath: outputPath };
    writeEvidence(outputPath, result);
    emit(result, argumentsValue.json);
  } catch (error) {
    const message = error instanceof Error ? error.message : String(error);
    writeEvidence(outputPath, {
      ok: false,
      command: "open",
      evidencePath: outputPath,
      doctor,
      error: message,
    });
    throw error;
  }
}

if (resolve(process.argv[1] ?? "") === fileURLToPath(import.meta.url)) {
  try {
    run();
  } catch (error) {
    const message = error instanceof Error ? error.message : String(error);
    process.stdout.write(`${JSON.stringify({ ok: false, code: "M0_PROOF_CLI_ERROR", message }, null, 2)}\n`);
    process.exitCode = 2;
  }
}

export const INTERNAL = Object.freeze({ PROJECT_ROOT, basename });
