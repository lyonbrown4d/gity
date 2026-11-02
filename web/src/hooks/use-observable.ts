import { useEffect, useRef, useSyncExternalStore } from "react";
import type { Observable } from "rxjs";

interface ObservableStore<T> {
  value: T;
  listeners: Set<() => void>;
}

/**
 * Subscribes to an RxJS observable and re-renders on each emission.
 *
 * The latest emission is read through `useSyncExternalStore`, so tearing is
 * avoided and the observable is only subscribed while the component is mounted.
 * Pass `null` while the source is not yet available; the hook then keeps
 * returning `initialValue` without subscribing.
 */
export function useObservable<T>(observable: Observable<T> | null, initialValue: T): T {
  const storeRef = useRef<ObservableStore<T>>({
    value: initialValue,
    listeners: new Set(),
  });

  const store = storeRef.current;
  store.value = store.value === initialValue ? initialValue : store.value;

  useEffect(() => {
    if (!observable) {
      return;
    }
    const subscription = observable.subscribe({
      next: (value) => {
        store.value = value;
        for (const listener of store.listeners) {
          listener();
        }
      },
    });
    return () => subscription.unsubscribe();
  }, [observable, store]);

  return useSyncExternalStore(
    (listener) => {
      store.listeners.add(listener);
      return () => {
        store.listeners.delete(listener);
      };
    },
    () => store.value,
    () => initialValue,
  );
}
