import { useState, useRef, useCallback, useEffect } from 'react';

export function useAudio() {
    const [isRecording, setIsRecording] = useState(false);
    const [volume, setVolume] = useState(0);
    const audioContextRef = useRef<AudioContext | null>(null);
    const analyzerRef = useRef<AnalyserNode | null>(null);
    const streamRef = useRef<MediaStream | null>(null);
    const animationFrameRef = useRef<number | null>(null);
    const mediaRecorderRef = useRef<MediaRecorder | null>(null);

    const startRecording = useCallback(async (onData: (base64: string) => void) => {
        try {
            const stream = await navigator.mediaDevices.getUserMedia({ audio: true });
            streamRef.current = stream;

            const audioContext = new AudioContext();
            audioContextRef.current = audioContext;

            const source = audioContext.createMediaStreamSource(stream);
            const analyzer = audioContext.createAnalyser();
            analyzer.fftSize = 256;
            source.connect(analyzer);
            analyzerRef.current = analyzer;

            const recorder = new MediaRecorder(stream);
            mediaRecorderRef.current = recorder;

            recorder.ondataavailable = async (e) => {
                if (e.data.size > 0) {
                    const reader = new FileReader();
                    reader.onloadend = () => {
                        const base64 = (reader.result as string).split(',')[1];
                        onData(base64);
                    };
                    reader.readAsDataURL(e.data);
                }
            };

            recorder.start(250); // Capture in 250ms chunks
            setIsRecording(true);

            const updateVolume = () => {
                const dataArray = new Uint8Array(analyzer.frequencyBinCount);
                analyzer.getByteFrequencyData(dataArray);
                const avg = dataArray.reduce((p, c) => p + c, 0) / dataArray.length;
                setVolume(avg / 255);
                animationFrameRef.current = requestAnimationFrame(updateVolume);
            };
            updateVolume();

        } catch (e) {
            console.error('Failed to start audio recording', e);
        }
    }, []);

    const stopRecording = useCallback(() => {
        mediaRecorderRef.current?.stop();
        streamRef.current?.getTracks().forEach(t => t.stop());
        if (animationFrameRef.current) cancelAnimationFrame(animationFrameRef.current);
        audioContextRef.current?.close();
        setIsRecording(false);
        setVolume(0);
    }, []);

    const playAudio = useCallback(async (base64: string) => {
        try {
            if (!audioContextRef.current || audioContextRef.current.state === 'closed') {
                audioContextRef.current = new AudioContext();
            }
            const ctx = audioContextRef.current;
            const arrayBuffer = Uint8Array.from(atob(base64), c => c.charCodeAt(0)).buffer;
            const audioBuffer = await ctx.decodeAudioData(arrayBuffer);
            const source = ctx.createBufferSource();
            source.buffer = audioBuffer;
            source.connect(ctx.destination);
            source.start();
        } catch (e) {
            console.error('Failed to play audio', e);
        }
    }, []);

    useEffect(() => {
        return () => {
            stopRecording();
        };
    }, [stopRecording]);

    return { isRecording, volume, startRecording, stopRecording, playAudio };
}
