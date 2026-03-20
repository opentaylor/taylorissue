import { useState, useEffect, useCallback } from "react";

const FRAME_COUNT = 145;
const FPS = 24;

export function HeroAnimation() {
  const [frame, setFrame] = useState(1);
  const [ready, setReady] = useState(false);
  const base = import.meta.env.BASE_URL;

  const getFrameUrl = useCallback(
    (n: number) =>
      `${base}hero-frames/frame_${String(n).padStart(4, "0")}.png`,
    [base]
  );

  useEffect(() => {
    let loaded = 0;
    for (let i = 1; i <= FRAME_COUNT; i++) {
      const img = new Image();
      img.src = getFrameUrl(i);
      img.onload = img.onerror = () => {
        loaded++;
        if (loaded >= FRAME_COUNT) setReady(true);
      };
    }
  }, [getFrameUrl]);

  useEffect(() => {
    if (!ready) return;
    const id = setInterval(() => {
      setFrame((f) => (f % FRAME_COUNT) + 1);
    }, 1000 / FPS);
    return () => clearInterval(id);
  }, [ready]);

  return (
    <img
      src={getFrameUrl(frame)}
      alt=""
      className="mx-auto w-full max-w-sm"
      loading="eager"
    />
  );
}
