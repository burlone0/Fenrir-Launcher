import { useState } from "react";
import { useUIStore } from "../stores/ui";

interface Props {
  onDone: () => void;
}

const TOTAL_STEPS = 4;

export default function OnboardingWizard({ onDone }: Props) {
  const [step, setStep] = useState(1);
  const navigate = useUIStore((s) => s.navigate);
  const openScan = useUIStore((s) => s.openScan);

  function dismiss() {
    localStorage.setItem("fenrir.onboarding_done", "true");
    onDone();
  }

  function advance() {
    if (step < TOTAL_STEPS) {
      setStep((s) => s + 1);
    } else {
      dismiss();
    }
  }

  function handlePrimary() {
    if (step === 2) {
      navigate("runtimes");
      advance();
    } else if (step === 3) {
      navigate("library");
      openScan();
      advance();
    } else {
      advance();
    }
  }

  const steps = [
    {
      title: "Welcome to Fenrir",
      body: "Your lightweight Linux game launcher. Let's get you set up in a few steps.",
      primary: "Get Started →",
      secondary: "Skip setup",
    },
    {
      title: "Install a Wine/Proton runtime",
      body: "Fenrir needs Wine-GE or GE-Proton to run games. You can install one from the Runtimes tab.",
      primary: "Open Runtimes",
      secondary: "Skip",
    },
    {
      title: "Scan your game directories",
      body: "Point Fenrir at a folder where your games live. Use Ctrl+S or the Scan button in the Library.",
      primary: "Open Library",
      secondary: "Skip",
    },
    {
      title: "You're all set!",
      body: "Your library will fill up as you scan. Fenrir will configure each game automatically.",
      primary: "Let's go",
      secondary: null,
    },
  ];

  const current = steps[step - 1];

  return (
    <div className="fixed inset-0 z-50 bg-black/70 flex items-center justify-center">
      <div className="bg-zinc-900 border border-zinc-700 rounded-xl w-full max-w-md p-8 flex flex-col gap-6">
        {/* Step dots */}
        <div className="flex items-center justify-center gap-2">
          {Array.from({ length: TOTAL_STEPS }, (_, i) => (
            <span
              key={i}
              className={`w-2 h-2 rounded-full ${
                i + 1 === step ? "bg-sky-500" : "bg-zinc-700"
              }`}
            />
          ))}
        </div>

        {/* Content */}
        <div className="flex flex-col gap-2">
          <h2 className="text-xl font-bold text-white">{current.title}</h2>
          <p className="text-sm text-zinc-400 leading-relaxed">{current.body}</p>
        </div>

        {/* Actions */}
        <div className="flex items-center justify-between mt-2">
          {current.secondary ? (
            <button
              className="text-zinc-500 hover:text-zinc-200 text-sm"
              onClick={dismiss}
            >
              {current.secondary}
            </button>
          ) : (
            <span />
          )}
          <button
            className="bg-sky-700 hover:bg-sky-600 text-white text-sm px-5 py-2 rounded font-medium"
            onClick={handlePrimary}
          >
            {current.primary}
          </button>
        </div>
      </div>
    </div>
  );
}
