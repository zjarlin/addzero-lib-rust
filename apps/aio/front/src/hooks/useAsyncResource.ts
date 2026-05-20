import {useCallback, useEffect, useRef, useState} from "react";

export interface AsyncResource<T> {
  data?: T;
  error?: string;
  loading: boolean;
  reload: () => Promise<void>;
  setData: (data: T) => void;
}

export function useAsyncResource<T>(loader: () => Promise<T>): AsyncResource<T> {
  const loaderRef = useRef(loader);
  const [data, setData] = useState<T>();
  const [error, setError] = useState<string>();
  const [loading, setLoading] = useState(true);
  loaderRef.current = loader;

  const reload = useCallback(async () => {
    setLoading(true);
    setError(undefined);
    try {
      setData(await loaderRef.current());
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    void reload();
  }, [reload]);

  return {data, error, loading, reload, setData};
}
