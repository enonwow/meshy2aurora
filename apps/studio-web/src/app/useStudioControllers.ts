import {
  useCallback,
  useEffect,
  useReducer,
  useRef,
} from "react";
import { StudioWorkerClient } from "../worker/client";
import {
  createInitialStudioSession,
  studioSessionReducer,
  type StudioSessionEvent,
  type StudioSessionState,
} from "./studioSession";

export function useStudioSessionController<
  TInspection,
  TResult,
  TAppearanceInspection,
>() {
  type State = StudioSessionState<
    TInspection,
    TResult,
    TAppearanceInspection
  >;
  type Event = StudioSessionEvent<
    TInspection,
    TAppearanceInspection,
    TResult
  >;

  const initialSessionRef = useRef<State | undefined>(undefined);
  if (!initialSessionRef.current) {
    initialSessionRef.current = createInitialStudioSession<
      TInspection,
      TResult,
      TAppearanceInspection
    >();
  }

  const [session, dispatch] = useReducer(
    (state: State, event: Event) => studioSessionReducer(state, event),
    initialSessionRef.current,
  );
  const sessionRef = useRef(session);
  sessionRef.current = session;

  return { session, sessionRef, dispatch };
}

export function useStudioWorkerController() {
  const workerRef = useRef<StudioWorkerClient | undefined>(undefined);

  useEffect(() => {
    const worker = new StudioWorkerClient();
    workerRef.current = worker;
    return () => {
      worker.dispose();
      if (workerRef.current === worker) workerRef.current = undefined;
    };
  }, []);

  const replaceWorker = useCallback(() => {
    workerRef.current?.dispose();
    workerRef.current = new StudioWorkerClient();
  }, []);

  return { workerRef, replaceWorker };
}
