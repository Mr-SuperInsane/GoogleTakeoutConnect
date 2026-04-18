import { useEffect } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { ask } from "@tauri-apps/plugin-dialog";

/**
 * 処理中にウィンドウを閉じようとした際に確認ダイアログを表示する
 */
export function useCloseGuard(isProcessing: boolean) {
  useEffect(() => {
    const win = getCurrentWindow();
    let unlisten: (() => void) | null = null;

    win.onCloseRequested(async (event) => {
      if (!isProcessing) return;

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

      if (confirmed) {
        await win.destroy();
      }
    }).then((fn) => {
      unlisten = fn;
    });

    return () => {
      unlisten?.();
    };
  }, [isProcessing]);
}
