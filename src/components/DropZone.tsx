import { useState, useCallback } from "react";
import { open } from "@tauri-apps/plugin-dialog";

interface Props {
  onStart: (zipPaths: string[], outputDir: string) => void;
}

export default function DropZone({ onStart }: Props) {
  const [zipPaths, setZipPaths] = useState<string[]>([]);
  const [outputDir, setOutputDir] = useState<string>("");
  const [isDragging, setIsDragging] = useState(false);

  const selectZips = async () => {
    const selected = await open({
      multiple: true,
      filters: [{ name: "ZIP", extensions: ["zip"] }],
      title: "Google TakeoutのZIPファイルを選択",
    });
    if (selected) {
      setZipPaths(Array.isArray(selected) ? selected : [selected]);
    }
  };

  const selectOutputDir = async () => {
    const selected = await open({
      directory: true,
      title: "出力先フォルダを選択",
    });
    if (selected && typeof selected === "string") {
      setOutputDir(selected);
    }
  };

  const handleDragOver = useCallback((e: React.DragEvent) => {
    e.preventDefault();
    setIsDragging(true);
  }, []);

  const handleDragLeave = useCallback(() => {
    setIsDragging(false);
  }, []);

  const handleDrop = useCallback((e: React.DragEvent) => {
    e.preventDefault();
    setIsDragging(false);
    const files = Array.from(e.dataTransfer.files)
      .filter((f) => f.name.endsWith(".zip"))
      .map((f) => f.name);
    if (files.length > 0) setZipPaths(files);
  }, []);

  const canStart = zipPaths.length > 0 && outputDir !== "";

  return (
    <div className="w-full max-w-xl flex flex-col gap-4">
      <div className="text-center mb-2">
        <h2 className="text-xl font-semibold text-gray-800">メタデータを復元する</h2>
        <p className="text-sm text-gray-500 mt-1">
          Google TakeoutのZIPファイルを選択して、撮影日時を写真・動画に書き込みます
        </p>
      </div>

      {/* ZIP選択エリア */}
      <div
        onClick={selectZips}
        onDragOver={handleDragOver}
        onDragLeave={handleDragLeave}
        onDrop={handleDrop}
        className={`
          border-2 border-dashed rounded-xl p-8 text-center cursor-pointer transition-colors
          ${isDragging
            ? "border-brand-500 bg-brand-50"
            : zipPaths.length > 0
              ? "border-brand-400 bg-brand-50"
              : "border-gray-300 bg-white hover:border-brand-400 hover:bg-gray-50"
          }
        `}
      >
        <div className="flex flex-col items-center gap-2">
          <div className={`w-12 h-12 rounded-full flex items-center justify-center ${
            zipPaths.length > 0 ? "bg-brand-100" : "bg-gray-100"
          }`}>
            <svg className={`w-6 h-6 ${zipPaths.length > 0 ? "text-brand-600" : "text-gray-400"}`}
              fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2}
                d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
            </svg>
          </div>
          {zipPaths.length > 0 ? (
            <div>
              <p className="font-medium text-brand-700">{zipPaths.length}個のZIPファイルを選択済み</p>
              <p className="text-xs text-gray-500 mt-1 max-w-xs truncate">
                {zipPaths.map((p) => p.split(/[/\\]/).pop()).join(", ")}
              </p>
              <p className="text-xs text-brand-500 mt-2">クリックして変更</p>
            </div>
          ) : (
            <div>
              <p className="font-medium text-gray-700">ZIPファイルを選択</p>
              <p className="text-xs text-gray-400 mt-1">クリックまたはドラッグ&ドロップ</p>
              <p className="text-xs text-gray-400">（複数選択可）</p>
            </div>
          )}
        </div>
      </div>

      {/* 出力先選択 */}
      <div
        onClick={selectOutputDir}
        className={`
          flex items-center gap-3 rounded-xl px-4 py-3 cursor-pointer border transition-colors
          ${outputDir
            ? "border-brand-400 bg-brand-50"
            : "border-gray-200 bg-white hover:border-brand-400 hover:bg-gray-50"
          }
        `}
      >
        <div className={`w-9 h-9 rounded-lg flex items-center justify-center flex-shrink-0 ${
          outputDir ? "bg-brand-100" : "bg-gray-100"
        }`}>
          <svg className={`w-5 h-5 ${outputDir ? "text-brand-600" : "text-gray-400"}`}
            fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2}
              d="M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-6l-2-2H5a2 2 0 00-2 2z" />
          </svg>
        </div>
        <div className="flex-1 min-w-0">
          <p className="text-sm font-medium text-gray-700">出力先フォルダ</p>
          {outputDir ? (
            <p className="text-xs text-brand-600 truncate">{outputDir}</p>
          ) : (
            <p className="text-xs text-gray-400">クリックして選択</p>
          )}
        </div>
      </div>

      {/* 対応フォーマット */}
      <div className="bg-gray-100 rounded-lg px-4 py-3">
        <p className="text-xs text-gray-500 font-medium mb-1">対応フォーマット</p>
        <div className="flex flex-wrap gap-1">
          {["JPG", "PNG", "HEIC", "WebP", "TIFF", "DNG", "CR2", "NEF", "ARW",
            "MP4", "MOV", "M4V", "3GP", "MKV"].map((ext) => (
            <span key={ext} className="text-xs bg-white text-gray-600 rounded px-1.5 py-0.5 border border-gray-200">
              {ext}
            </span>
          ))}
        </div>
      </div>

      {/* 開始ボタン */}
      <button
        onClick={() => canStart && onStart(zipPaths, outputDir)}
        disabled={!canStart}
        className={`
          w-full py-3 rounded-xl font-semibold text-sm transition-all
          ${canStart
            ? "bg-brand-600 text-white hover:bg-brand-700 active:scale-[0.98] shadow-sm"
            : "bg-gray-200 text-gray-400 cursor-not-allowed"
          }
        `}
      >
        メタデータを復元する
      </button>
    </div>
  );
}
