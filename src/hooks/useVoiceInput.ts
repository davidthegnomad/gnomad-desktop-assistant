import { useCallback, useEffect, useRef, useState } from "react";
import { getSpeechRecognition } from "../lib/voiceInput";

interface UseVoiceInputOptions {
  enabled: boolean;
  onFinalTranscript: (text: string) => void;
}

/** Push-to-talk via Web Speech API — click mic to start/stop dictation. */
export function useVoiceInput({ enabled, onFinalTranscript }: UseVoiceInputOptions) {
  const [listening, setListening] = useState(false);
  const [supported] = useState(() => getSpeechRecognition() !== null);
  const recRef = useRef<SpeechRecognition | null>(null);
  const onFinalRef = useRef(onFinalTranscript);
  onFinalRef.current = onFinalTranscript;

  useEffect(() => {
    const SR = getSpeechRecognition();
    if (!SR || !enabled) {
      recRef.current = null;
      return;
    }

    const rec = new SR();
    rec.continuous = false;
    rec.interimResults = true;
    rec.lang = navigator.language || "en-US";

    rec.onresult = (event) => {
      let final = "";
      for (let i = event.resultIndex; i < event.results.length; i++) {
        if (event.results[i].isFinal) {
          final += event.results[i][0].transcript;
        }
      }
      const trimmed = final.trim();
      if (trimmed) onFinalRef.current(trimmed);
    };

    rec.onend = () => setListening(false);
    rec.onerror = () => setListening(false);
    recRef.current = rec;

    return () => {
      try {
        rec.abort();
      } catch {
        /* already stopped */
      }
      recRef.current = null;
    };
  }, [enabled]);

  const toggle = useCallback(() => {
    const rec = recRef.current;
    if (!rec || !enabled) return;
    if (listening) {
      rec.stop();
      setListening(false);
      return;
    }
    try {
      rec.start();
      setListening(true);
    } catch {
      setListening(false);
    }
  }, [enabled, listening]);

  return { listening, supported, toggle };
}
