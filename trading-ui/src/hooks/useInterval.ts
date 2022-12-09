import { useEffect, useRef } from 'react';

export function useInterval(millis: number, callback: () => Promise<void>) {
  const ref = useRef(callback);

  useEffect(() => {
    ref.current = callback;
  });

  useEffect(() => {
    const interval = setInterval(() => {
      ref.current();
    }, millis);

    return () => {
      clearInterval(interval);
    };
  }, [millis]);
}
