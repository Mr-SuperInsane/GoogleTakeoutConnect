import { useEffect } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { ask } from "@tauri-apps/plugin-dialog";

export function useCloseGuard(isProcessing: boolean) {
  useEffect(() => {
    if (!isProcessing) return; // 処理中以外はハンドラ不要

    const win = getCurrentWindow();
    let unlisten: (() => void) | null = null;
    let cancelled = false;

    win.onCloseRequested(async (event) => {
      event.preventDefault();
      const confirmed = await ask(
        "処理がまだ完了していません。\nアプリを閉じると処理が中断されます。\n\n本当に閉じますか？",
        {
          title: "処理中です",
          kind: "warning",
          okLabel: "閉じる",
          cancelLabel: "キャンセル",
        }
      );
      if (confirmed) await win.destroy();
    }).then((fn) => {
      if (cancelled) fn(); // クリーンアップ後に Promise が解決した場合は即解除
      else unlisten = fn;
    });

    return () => {
      cancelled = true;
      unlisten?.();
    };
  }, [isProcessing]);
}
