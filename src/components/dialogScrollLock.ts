let openDialogs = 0;
let restoreDocument: (() => void) | undefined;

export function lockDocumentScroll() {
  if (openDialogs === 0) {
    const root = document.documentElement;
    const overflow = root.style.overflow;
    const { scrollX, scrollY } = window;
    root.style.overflow = 'hidden';
    restoreDocument = () => {
      root.style.overflow = overflow;
      window.scrollTo(scrollX, scrollY);
    };
  }
  openDialogs++;
  return () => {
    openDialogs--;
    if (openDialogs === 0) {
      restoreDocument?.();
      restoreDocument = undefined;
    }
  };
}
