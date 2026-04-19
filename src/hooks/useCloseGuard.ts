import { useEffect, useRef } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { ask } from "@tauri-apps/plugin-dialog";

export function useCloseGuard(isProcessing: boolean) {
  const isProcessingRef = useRef(isProcessing);

  useEffect(() => {
    isProcessingRef.current = isProcessing;
  }, [isProcessing]);

  useEffect(() => {
    const win = getCurrentWindow();
    let unlisten: (() => void) | null = null;

    win.onCloseRequested(async (event) => {
      if (!isProcessingRef.current) return;

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
  }, []); // リスナーは一度だけ登録
}
