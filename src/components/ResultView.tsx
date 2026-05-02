import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { AppState, LogEntry, ProcessResult } from "../App";

interface Props {
  result: ProcessResult;
  state: AppState;
  logs: LogEntry[];
  onReset: () => void;
}

type FilterStatus = "all" | "skipped" | "failed";

export default function ResultView({ result, state, logs, onReset }: Props) {
  const [filter, setFilter] = useState<FilterStatus>("all");
  const isSuccess = state === "done";

  const openOutputDir = async () => {
    await invoke("open_directory", { path: result.outputDir });
  };

  const filteredLogs = filter === "all"
    ? logs
    : logs.filter((l) => l.status === filter);

  const skipTotal = result.skipReasons.no_json
    + result.skipReasons.no_timestamp
    + result.skipReasons.parse_error;

  return (
    <div className="w-full max-w-2xl flex flex-col gap-4">
      {/* ステータスヘッダー */}
      <div className={`rounded-xl p-4 flex items-center gap-4 ${
        isSuccess ? "bg-green-50 border border-green-200" : "bg-red-50 border border-red-200"
      }`}>
        <div className={`w-10 h-10 rounded-full flex items-center justify-center flex-shrink-0 ${
          isSuccess ? "bg-green-100" : "bg-red-100"
        }`}>
          {isSuccess ? (
            <svg className="w-5 h-5 text-green-600" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M5 13l4 4L19 7" />
            </svg>
          ) : (
            <svg className="w-5 h-5 text-red-600" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
            </svg>
          )}
        </div>
        <div>
          <p className={`font-semibold ${isSuccess ? "text-green-800" : "text-red-800"}`}>
            {isSuccess ? "処理が完了しました" : "処理中にエラーが発生しました"}
          </p>
          <p className={`text-sm mt-0.5 ${isSuccess ? "text-green-600" : "text-red-600"}`}>
            {result.total}件中 {result.success}件成功
          </p>
        </div>
      </div>

      {/* カウンター */}
      <div className="grid grid-cols-3 gap-3">
        <StatCard label="成功" value={result.success} color="green" />
        <StatCard label="スキップ" value={result.skipped} color="yellow" />
        <StatCard label="失敗" value={result.failed} color="red" />
      </div>

      {/* スキップ理由の内訳 */}
      {skipTotal > 0 && (
        <div className="bg-yellow-50 border border-yellow-200 rounded-xl px-4 py-3">
          <p className="text-sm font-semibold text-yellow-800 mb-2">
            スキップの理由（計{skipTotal}件）
          </p>
          <div className="space-y-1">
            {result.skipReasons.no_json > 0 && (
              <SkipRow
                count={result.skipReasons.no_json}
                label="JSONメタデータが見つからなかった"
                detail="Google Takeoutのエクスポート時にメタデータが含まれていないファイルです。元の撮影日時が不明のためスキップしました。"
              />
            )}
            {result.skipReasons.no_timestamp > 0 && (
              <SkipRow
                count={result.skipReasons.no_timestamp}
                label="JSONにタイムスタンプが含まれていない"
                detail="JSONファイルは存在しますが、撮影日時の情報が記録されていませんでした。"
              />
            )}
            {result.skipReasons.parse_error > 0 && (
              <SkipRow
                count={result.skipReasons.parse_error}
                label="JSONの解析に失敗"
                detail="JSONファイルの形式が正しくないためメタデータを読み取れませんでした。"
              />
            )}
          </div>
          <p className="text-xs text-yellow-700 mt-2">
            ※ スキップされたファイルはそのまま出力フォルダにコピーされています
          </p>
        </div>
      )}

      {/* ログビューア */}
      {logs.length > 0 && (
        <div className="bg-gray-900 rounded-xl overflow-hidden">
          <div className="px-3 py-2 border-b border-gray-700 flex items-center gap-2">
            <span className="text-xs text-gray-400 font-mono">処理ログ</span>
            <div className="ml-auto flex gap-1">
              {(["all", "skipped", "failed"] as FilterStatus[]).map((s) => (
                <button
                  key={s}
                  onClick={() => setFilter(s)}
                  className={`text-xs px-2 py-0.5 rounded transition-colors ${
                    filter === s
                      ? "bg-gray-600 text-white"
                      : "text-gray-400 hover:text-gray-200"
                  }`}
                >
                  {s === "all" ? `全て(${logs.length})` :
                   s === "skipped" ? `スキップ(${logs.filter(l => l.status === "skipped").length})` :
                   `失敗(${logs.filter(l => l.status === "failed").length})`}
                </button>
              ))}
            </div>
          </div>
          <div className="h-44 overflow-y-auto px-3 py-2 font-mono text-xs space-y-0.5">
            {filteredLogs.map((log, i) => (
              <div key={i} className="flex items-start gap-2 leading-5">
                <span className={`flex-shrink-0 font-bold ${
                  log.status === "success" ? "text-green-500" :
                  log.status === "skipped" ? "text-yellow-500" : "text-red-500"
                }`}>
                  {log.status === "success" ? "✓" : log.status === "skipped" ? "–" : "✕"}
                </span>
                <span className="text-gray-300 truncate flex-1">{log.file}</span>
                {log.status !== "success" && (
                  <span className={`flex-shrink-0 text-right text-xs ${
                    log.status === "skipped" ? "text-yellow-500" : "text-red-500"
                  }`}>
                    {log.message}
                  </span>
                )}
              </div>
            ))}
          </div>
        </div>
      )}

      {/* アクションボタン */}
      <div className="flex gap-3">
        {isSuccess && result.outputDir && (
          <button
            onClick={openOutputDir}
            className="flex-1 py-3 rounded-xl font-semibold text-sm bg-brand-600 text-white hover:bg-brand-700 active:scale-[0.98] transition-all shadow-sm"
          >
            出力フォルダを開く
          </button>
        )}
        <button
          onClick={onReset}
          className="flex-1 py-3 rounded-xl font-semibold text-sm border border-gray-300 bg-white text-gray-700 hover:bg-gray-50 transition-all"
        >
          もう一度処理する
        </button>
      </div>
    </div>
  );
}

function StatCard({ label, value, color }: { label: string; value: number; color: string }) {
  const colors: Record<string, string> = {
    green: "bg-green-50 border-green-200 text-green-700",
    yellow: "bg-yellow-50 border-yellow-200 text-yellow-700",
    red: "bg-red-50 border-red-200 text-red-700",
  };
  return (
    <div className={`rounded-lg border px-4 py-3 text-center ${colors[color]}`}>
      <p className="text-2xl font-bold">{value}</p>
      <p className="text-xs mt-0.5">{label}</p>
    </div>
  );
}

function SkipRow({ count, label, detail }: { count: number; label: string; detail: string }) {
  const [open, setOpen] = useState(false);
  return (
    <div>
      <button
        onClick={() => setOpen(!open)}
        className="flex items-center gap-2 w-full text-left text-sm text-yellow-700 hover:text-yellow-900"
      >
        <span className="font-semibold">{count}件</span>
        <span>{label}</span>
        <span className="ml-auto text-yellow-500">{open ? "▲" : "▼"}</span>
      </button>
      {open && (
        <p className="text-xs text-yellow-600 mt-1 ml-6 leading-relaxed">{detail}</p>
      )}
    </div>
  );
}
