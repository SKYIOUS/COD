"""
COD vs VS Code vs Zed — automated benchmark.

Requires: pywinauto, psutil, hyperfine

Usage:
  python benchmark_editors.py                   # 5 iterations each
  python benchmark_editors.py --quick            # 2 iterations
  python benchmark_editors.py --hyperfine        # CLI startup only
"""

import os, sys, time, json, tempfile, shutil, subprocess, statistics, argparse
from pathlib import Path

EDITORS = {
    "COD": {
        "exe":   R"C:\Program Files\COD Editor\COD.exe",
        "flags": ["--disable-extensions", "--skip-release-notes"],
        "titles": ["COD"],
    },
    "VS Code": {
        "exe":   R"C:\Users\nanda\AppData\Local\Programs\Microsoft VS Code\Code.exe",
        "flags": ["--disable-extensions", "--skip-release-notes"],
        "titles": ["Visual Studio Code", "Code"],
    },
    "Zed": {
        "exe":   R"C:\Users\nanda\AppData\Local\Programs\Zed\zed.exe",
        "flags": [],
        "titles": ["Zed"],
    },
}

WORKSPACE = Path(__file__).parent / ".bench_workspace"
ITERS = 5
QUICK = 2
WARMUP = 1


def prep_workspace():
    WORKSPACE.mkdir(exist_ok=True)
    (WORKSPACE / "readme.md").write_text("# COD Benchmark\n")


def kill_all():
    import psutil
    targets = {"cod.exe", "code.exe", "zed.exe"}
    for p in psutil.process_iter(["pid", "name"]):
        try:
            if p.info["name"] and p.info["name"].lower() in targets:
                p.kill()
        except (psutil.NoSuchProcess, psutil.AccessDenied):
            pass
    time.sleep(2)


def descendants(pid: int) -> set:
    import psutil
    pids = {pid}
    try:
        for c in psutil.Process(pid).children(recursive=True):
            pids.add(c.pid)
    except (psutil.NoSuchProcess, psutil.AccessDenied):
        pass
    return pids


def measure_memory(pids: set) -> dict:
    import psutil
    priv, wset = [], []
    for _ in range(3):
        p, w = 0, 0
        for pid in pids:
            try:
                mi = psutil.Process(pid).memory_info()
                p += mi.private or 0
                w += mi.rss or 0
            except (psutil.NoSuchProcess, psutil.AccessDenied):
                pass
        priv.append(p); wset.append(w)
        time.sleep(0.2)
    return {
        "mem_private_mb": round(statistics.median(priv) / 1_048_576, 1),
        "mem_wset_mb":    round(statistics.median(wset) / 1_048_576, 1),
    }


def detect_window(root_pid: int, titles: list, t0: float, timeout=25):
    import win32gui, win32process
    pids = descendants(root_pid)
    end = t0 + timeout
    while time.perf_counter() < end:
        pids.update(descendants(root_pid))
        found = []
        def cb(hwnd, _):
            if win32gui.IsWindowVisible(hwnd):
                _, wpid = win32process.GetWindowThreadProcessId(hwnd)
                text = win32gui.GetWindowText(hwnd)
                if wpid in pids and text:
                    found.append((wpid, text))
        win32gui.EnumWindows(cb, None)
        if found:
            for wpid, text in found:
                for t in titles:
                    if t.lower() in text.lower():
                        return time.perf_counter() - t0
            return time.perf_counter() - t0
        time.sleep(0.05)
    return None


def detect_window_uia(pids: set, titles: list, t0: float, timeout=25):
    try:
        from pywinauto import Desktop
        from pywinauto.timings import wait_until
    except ImportError:
        return None
    desktop = Desktop(backend="uia")
    end = t0 + timeout
    def cond():
        if time.perf_counter() >= end:
            return True
        pids.update(descendants(min(pids)))
        for w in desktop.windows():
            try:
                if w.is_visible():
                    t = w.window_text()
                    if not t:
                        continue
                    wp = w.process_id() if hasattr(w, 'process_id') else None
                    if (wp and wp in pids) or any(x.lower() in t.lower() for x in titles):
                        return True
            except Exception:
                pass
        return False
    try:
        wait_until(timeout, 0.05, cond)
        return time.perf_counter() - t0
    except Exception:
        return None


def measure_once(name: str):
    cfg = EDITORS[name]
    userdir = tempfile.mkdtemp(prefix=f"bench_{name.lower().replace(' ','_')}_")
    args = [cfg["exe"]] + cfg["flags"] + ["--user-data-dir", userdir, str(WORKSPACE)]

    t0 = time.perf_counter()
    proc = subprocess.Popen(args, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
    pid = proc.pid
    pids = {pid}

    elapsed = detect_window(pid, cfg["titles"], t0)
    if elapsed is None:
        elapsed = detect_window_uia({pid}, cfg["titles"], t0)

    if elapsed is not None:
        launch_ms = (time.perf_counter() - t0) * 1000
        time.sleep(2)
        mem = measure_memory(descendants(pid))
    else:
        launch_ms = (time.perf_counter() - t0) * 1000
        mem = {"mem_private_mb": 0, "mem_wset_mb": 0}
        elapsed = 25.0

    try:
        proc.kill()
        proc.wait(timeout=5)
    except Exception:
        pass
    time.sleep(1)
    shutil.rmtree(userdir, ignore_errors=True)

    return {"editor": name, "launch_ms": round(launch_ms, 1),
            "startup_ms": round(elapsed * 1000, 1), **mem}


HF_RUNNER_TEMPLATE = r"""import subprocess, time, win32gui, win32process, sys, os, json

EXE = {exe!r}
FLAGS = {flags!r}
TITLES = {titles!r}
WS = {ws!r}
TD = {td!r}

args = [EXE] + FLAGS + ["--user-data-dir", TD, WS]
t0 = time.perf_counter()
proc = subprocess.Popen(args, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)

def descendants(pid):
    import psutil
    pids = set([pid])
    try:
        for c in psutil.Process(pid).children(recursive=True):
            pids.add(c.pid)
    except Exception:
        pass
    return pids

pids = descendants(proc.pid)
end = t0 + 25
found = None
while time.perf_counter() < end:
    pids.update(descendants(proc.pid))
    matched = []
    def cb(hwnd, _):
        if win32gui.IsWindowVisible(hwnd):
            _, wpid = win32process.GetWindowThreadProcessId(hwnd)
            text = win32gui.GetWindowText(hwnd)
            if wpid in pids and text:
                matched.append((time.perf_counter() - t0, text))
    win32gui.EnumWindows(cb, None)
    if matched:
        for elapsed, text in matched:
            for t in TITLES:
                if t.lower() in text.lower():
                    found = elapsed
                    break
            if found:
                break
        if not found:
            found = matched[0][0]
    if found:
        break
    time.sleep(0.05)

elapsed = found if found else 25.0
try:
    proc.kill()
    proc.wait(timeout=5)
except Exception:
    pass
print(elapsed)
"""

def _make_hf_runner(name: str, userdir: str) -> str:
    cfg = EDITORS[name]
    script = HF_RUNNER_TEMPLATE.format(
        exe=cfg["exe"],
        flags=cfg["flags"],
        titles=cfg["titles"],
        ws=str(WORKSPACE),
        td=userdir,
    )
    path = Path(userdir) / "_hf_runner.py"
    path.write_text(script)
    return str(path)


_HF_RUNS = 10
_HF_WARMUP = 2

def set_hf_params(runs: int, warmup: int):
    global _HF_RUNS, _HF_WARMUP
    _HF_RUNS = runs
    _HF_WARMUP = warmup

def hyperfine_cmdline(name: str) -> tuple[list[str], str]:
    """Return (hyperfine_argv, shell_command) for dry-run display."""
    cfg = EDITORS[name]
    userdir = tempfile.mkdtemp(prefix=f"hf_{name.lower().replace(' ','_')}_")
    runner = _make_hf_runner(name, userdir)
    shell_cmd = f"python \"{runner}\""
    argv = ["hyperfine", "--warmup", str(_HF_WARMUP), "--runs", str(_HF_RUNS),
            "--command-name", name,
            "--export-json", f"{userdir}\\hf.json",
            "--show-output", shell_cmd]
    return argv, shell_cmd


def run_hyperfine(name: str, dry_run: bool = False):
    cfg = EDITORS[name]
    argv, shell_cmd = hyperfine_cmdline(name)
    if dry_run:
        print(" ".join(argv))
        return None
    try:
        r = subprocess.run(argv, capture_output=True, text=True, timeout=120)
        # extract the export-json path from argv
        idx = argv.index("--export-json") + 1
        json_path = argv[idx]
        with open(json_path) as f:
            return json.load(f)["results"][0]
    except Exception as e:
        print(f"    [hyperfine error] {e}")
        return None
    finally:
        # cleanup tempdir — extract from argv
        for a in argv:
            if a.startswith("C:\\") and "_hf_runner.py" in a:
                d = Path(a).parent
                if d.exists():
                    shutil.rmtree(d, ignore_errors=True)
                break


def report(all_results):
    print("\n" + "=" * 64)
    print("  RESULTS")
    print("=" * 64)
    h = f"{'Editor':<12} {'Window(ms)':<17} {'Private(MB)':<17} {'WorkingSet(MB)':<17}"
    print(f"\n{h}\n" + "-" * len(h))

    for name in EDITORS:
        vals = [r for r in all_results if r["editor"] == name]
        if not vals:
            continue
        def mu(k):
            return statistics.mean([v[k] for v in vals])
        def sd(k):
            return statistics.stdev([v[k] for v in vals]) if len(vals) > 1 else 0
        print(f"{name:<12} {mu('startup_ms'):>6.0f} +-{sd('startup_ms'):>4.0f}    "
              f"{mu('mem_private_mb'):>5.1f} +-{sd('mem_private_mb'):>4.1f}     "
              f"{mu('mem_wset_mb'):>5.1f} +-{sd('mem_wset_mb'):>4.1f}")

    cod = [r for r in all_results if r["editor"] == "COD"]
    vsc = [r for r in all_results if r["editor"] == "VS Code"]
    zed = [r for r in all_results if r["editor"] == "Zed"]

    if cod and vsc:
        cs = statistics.mean([r["startup_ms"] for r in cod])
        cm = statistics.mean([r["mem_private_mb"] for r in cod])
        vs = statistics.mean([r["startup_ms"] for r in vsc])
        vm = statistics.mean([r["mem_private_mb"] for r in vsc])
        print(f"\n  COD vs VS Code:  startup {vs/cs:.2f}x,  memory {vm/cm:.2f}x")
    if cod and zed:
        cs = statistics.mean([r["startup_ms"] for r in cod])
        cm = statistics.mean([r["mem_private_mb"] for r in cod])
        zs = statistics.mean([r["startup_ms"] for r in zed])
        zm = statistics.mean([r["mem_private_mb"] for r in zed])
        print(f"  COD vs Zed:      startup {zs/cs:.2f}x,  memory {zm/cm:.2f}x")

    out = Path(__file__).parent / ".bench_results.json"
    out.write_text(json.dumps(all_results, indent=2), encoding="utf-8")
    print(f"\nRaw results -> {out}")

    print("\n--- MD snippet ---")
    for name in EDITORS:
        vals = [r for r in all_results if r["editor"] == name]
        if not vals:
            continue
        print(f"| {name:<10} | {statistics.mean([v['startup_ms'] for v in vals]):>6.0f} ms"
              f" | {statistics.mean([v['mem_private_mb'] for v in vals]):>5.1f} MB"
              f" | {statistics.mean([v['mem_wset_mb'] for v in vals]):>5.1f} MB |")


def _resolve_editors(names: str | None) -> list[str]:
    if not names:
        return list(EDITORS)
    selected = [n.strip() for n in names.split(",")]
    unknown = [n for n in selected if n not in EDITORS]
    if unknown:
        print(f"Unknown editors: {unknown}. Available: {list(EDITORS.keys())}")
        sys.exit(1)
    return selected


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--quick", action="store_true")
    ap.add_argument("--hyperfine", action="store_true")
    ap.add_argument("--hf-runs", type=int, default=10, help="hyperfine runs (default: 10)")
    ap.add_argument("--hf-warmup", type=int, default=2, help="hyperfine warmup (default: 2)")
    ap.add_argument("-e", "--editor", help="comma-separated editors (default: all)")
    ap.add_argument("--dry-run", action="store_true", help="print hyperfine commands without running")
    ap.add_argument("--list-editors", action="store_true", help="list available editors and exit")
    args = ap.parse_args()

    if args.list_editors:
        print("Available editors:")
        for name, cfg in EDITORS.items():
            print(f"  {name:<12} {cfg['exe']}")
        return

    if args.dry_run and not args.hyperfine:
        print("--dry-run requires --hyperfine")
        sys.exit(1)

    editors = _resolve_editors(args.editor)
    iters = QUICK if args.quick else ITERS

    print("=" * 64)
    print("  COD vs VS Code vs Zed - Editor Benchmark")
    print("=" * 64)
    print(f"  Date:   {time.strftime('%Y-%m-%d %H:%M')}")
    print(f"  Editors: {', '.join(editors)}")
    if args.hyperfine:
        print(f"  HF:     {args.hf_runs} runs (+{args.hf_warmup} warmup)")
        if args.dry_run:
            print(f"  Mode:   DRY RUN (commands printed, nothing executed)")
    else:
        print(f"  Iters:  {iters}  (+{WARMUP} warmup)")
    print(f"  WS:     {WORKSPACE}\n")

    prep_workspace()

    if args.hyperfine:
        set_hf_params(args.hf_runs, args.hf_warmup)
        if args.dry_run:
            print("--- hyperfine: DRY RUN ---")
            for name in editors:
                argv, _ = hyperfine_cmdline(name)
                print(f"  # {name}")
                print(f"  {' '.join(argv)}")
                print()
            return
        print("--- hyperfine ---")
        hf = {}
        for name in editors:
            print(f"  {name} ... ", end="", flush=True)
            kill_all(); time.sleep(1)
            r = run_hyperfine(name)
            if r:
                hf[name] = r
                print(f"{r['mean']*1000:.0f}ms  [{r['min']*1000:.0f}-{r['max']*1000:.0f}]")
            else:
                print("skip")
        if hf:
            print(f"\n{'Editor':<12} {'Mean(ms)':<12} {'Min(ms)':<12} {'Max(ms)':<10}")
            print("-" * 46)
            for n, r in hf.items():
                print(f"{n:<12} {r['mean']*1000:>6.0f}      {r['min']*1000:>6.0f}      {r['max']*1000:>6.0f}")
        return

    all_results = []
    for name in editors:
        print(f"--- {name} ---")
        for _ in range(WARMUP):
            print("  warmup ... ", end="", flush=True)
            r = measure_once(name)
            print(f"window {r['startup_ms']}ms, priv {r['mem_private_mb']}MB")
        for i in range(iters):
            print(f"  run {i+1}/{iters} ... ", end="", flush=True)
            kill_all(); time.sleep(1)
            r = measure_once(name)
            all_results.append(r)
            print(f"window {r['startup_ms']}ms, "
                  f"priv {r['mem_private_mb']}MB, "
                  f"wset {r['mem_wset_mb']}MB")
            time.sleep(1)

    report(all_results)


if __name__ == "__main__":
    main()
