import { useState } from "react";
import appIcon from "./assets/icon.png";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import DropZone from "./components/DropZone";
import ProcessingView from "./components/ProcessingView";
import ResultView from "./components/ResultView";
import { useCloseGuard } from "./hooks/useCloseGuard";

export type AppState = "idle" | "processing" | "done" | "error";

export interface LogEntry {
  file: string;
  status: "success" | "skipped" | "failed";
  message: string;
}

export interface SkipReasonSummary {
  no_json: number;
  no_timestamp: number;
  parse_error: number;
}

export interface ProcessResult {
  total: number;
  success: number;
  skipped: number;
  failed: number;
  outputDir: string;
  skipReasons: SkipReasonSummary;
  errors: string[];
}

export interface Progress {
  current: number;
  total: number;
  success: number;
  skipped: number;
  failed: number;
  currentFile: string;
}

function App() {
  const [state, setState] = useState<AppState>("idle");
  const [result, setResult] = useState<ProcessResult | null>(null);
  useCloseGuard(state === "processing");
  const [progress, setProgress] = useState<Progress>({
    current: 0, total: 0, success: 0, skipped: 0, failed: 0, currentFile: "",
  });
  const [logs, setLogs] = useState<LogEntry[]>([]);

  const handleStart = async (zipPaths: string[], outputDir: string) => {
    setState("processing");
    setLogs([]);
    setProgress({ current: 0, total: 0, success: 0, skipped: 0, failed: 0, currentFile: "" });

    const unlistenProgress = await listen<Progress>("progress", (e) => {
      setProgress(e.payload);
    });
    const unlistenLog = await listen<LogEntry>("log", (e) => {
      setLogs((prev) => {
        const next = [...prev, e.payload];
        // 最大500件保持
        return next.length > 500 ? next.slice(next.length - 500) : next;
      });
    });

    try {
      const res = await invoke<{
        total: number; success: number; skipped: number; failed: number;
        output_dir: string; skip_reasons: SkipReasonSummary; errors: string[];
      }>("process_takeout", { zipPaths, outputDir });

      setResult({
        total: res.total,
        success: res.success,
        skipped: res.skipped,
        failed: res.failed,
        outputDir: res.output_dir,
        skipReasons: res.skip_reasons,
        errors: res.errors,
      });
      setState("done");
    } catch (e) {
      setResult({
        total: 0, success: 0, skipped: 0, failed: 0,
        outputDir: "",
        skipReasons: { no_json: 0, no_timestamp: 0, parse_error: 0 },
        errors: [String(e)],
      });
      setState("error");
    } finally {
      unlistenProgress();
      unlistenLog();
    }
  };

  const handleReset = () => {
    setState("idle");
    setResult(null);
    setLogs([]);
    setProgress({ current: 0, total: 0, success: 0, skipped: 0, failed: 0, currentFile: "" });
  };

  return (
    <div className="min-h-screen flex flex-col">
      <Header />
      <main className="flex-1 flex flex-col items-center justify-center p-6">
        {state === "idle" && <DropZone onStart={handleStart} />}
        {state === "processing" && <ProcessingView progress={progress} logs={logs} />}
        {(state === "done" || state === "error") && result && (
          <ResultView result={result} state={state} logs={logs} onReset={handleReset} />
        )}
      </main>
    </div>
  );
}

function Header() {
  return (
    <header className="bg-white border-b border-gray-200 px-6 py-4 flex items-center gap-3">
      <img src={appIcon} alt="icon" className="w-8 h-8 rounded-lg" />
      <div>
        <h1 className="text-lg font-bold text-gray-900 leading-none">Google Takeout Connect</h1>
        <p className="text-xs text-gray-500 mt-0.5">写真・動画のメタデータを復元する</p>
      </div>
    </header>
  );
}

export default App;
