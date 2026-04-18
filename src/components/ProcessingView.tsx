import { useEffect, useRef } from "react";
import type { LogEntry, Progress } from "../App";

interface Props {
  progress: Progress;
  logs: LogEntry[];
}

const STATUS_ICON: Record<string, string> = {
  success: "✓",
  skipped: "–",
  failed: "✕",
};

const STATUS_COLOR: Record<string, string> = {
  success: "text-green-600",
  skipped: "text-yellow-600",
  failed: "text-red-600",
};

export default function ProcessingView({ progress, logs }: Props) {
  const logRef = useRef<HTMLDivElement>(null);
  const pct = progress.total > 0
    ? Math.round((progress.current / progress.total) * 100)
    : 0;

  // 新しいログが追加されたら自動スクロール
  useEffect(() => {
    if (logRef.current) {
      logRef.current.scrollTop = logRef.current.scrollHeight;
    }
  }, [logs]);

  return (
    <div className="w-full max-w-2xl flex flex-col gap-4">
      {/* ヘッダー */}
      <div className="flex items-center gap-3">
        <div className="w-6 h-6 rounded-full border-2 border-brand-200 border-t-brand-600 animate-spin flex-shrink-0" />
        <div>
          <p className="font-semibold text-gray-800">処理中...</p>
          <p className="text-xs text-gray-500">
            {progress.total > 0
              ? `${progress.current} / ${progress.total} ファイル`
              : "ファイルを解析中"}
          </p>
        </div>
      </div>

      {/* プログレスバー */}
      {progress.total > 0 && (
        <div>
          <div className="flex justify-between text-xs text-gray-500 mb-1">
            <span>{pct}%</span>
            <span>{progress.total}件</span>
          </div>
          <div className="w-full bg-gray-200 rounded-full h-2">
            <div
              className="bg-brand-600 h-2 rounded-full transition-all duration-200"
              style={{ width: `${pct}%` }}
            />
          </div>
        </div>
      )}

      {/* リアルタイムカウンター */}
      {progress.total > 0 && (
        <div className="grid grid-cols-3 gap-2 text-center text-sm">
          <div className="bg-green-50 border border-green-200 rounded-lg py-2">
            <p className="text-lg font-bold text-green-700">{progress.success}</p>
            <p className="text-xs text-green-600">成功</p>
          </div>
          <div className="bg-yellow-50 border border-yellow-200 rounded-lg py-2">
            <p className="text-lg font-bold text-yellow-700">{progress.skipped}</p>
            <p className="text-xs text-yellow-600">スキップ</p>
          </div>
          <div className="bg-red-50 border border-red-200 rounded-lg py-2">
            <p className="text-lg font-bold text-red-700">{progress.failed}</p>
            <p className="text-xs text-red-600">失敗</p>
          </div>
        </div>
      )}

      {/* ログパネル */}
      <div className="bg-gray-900 rounded-xl overflow-hidden">
        <div className="px-3 py-2 border-b border-gray-700 flex items-center gap-2">
          <span className="w-2 h-2 rounded-full bg-green-400 animate-pulse" />
          <span className="text-xs text-gray-400 font-mono">処理ログ</span>
          <span className="ml-auto text-xs text-gray-500">{logs.length}件</span>
        </div>
        <div
          ref={logRef}
          className="h-52 overflow-y-auto px-3 py-2 font-mono text-xs space-y-0.5"
        >
          {logs.length === 0 ? (
            <p className="text-gray-600 py-2">ログを待機中...</p>
          ) : (
            logs.slice().reverse().map((log, i) => (
              <div key={i} className="flex items-start gap-2 leading-5">
                <span className={`flex-shrink-0 font-bold ${STATUS_COLOR[log.status]}`}>
                  {STATUS_ICON[log.status]}
                </span>
                <span className="text-gray-300 truncate flex-1">{log.file}</span>
                {log.status !== "success" && (
                  <span className={`flex-shrink-0 text-right ${STATUS_COLOR[log.status]}`}>
                    {log.message}
                  </span>
                )}
              </div>
            ))
          )}
        </div>
      </div>

      <p className="text-xs text-gray-400 text-center">アプリを閉じないでください</p>
    </div>
  );
}
