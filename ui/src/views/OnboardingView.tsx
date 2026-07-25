import { useState } from 'react';
import { Button } from '@/components/ui/Button';
import { Card, CardContent } from '@/components/ui/Card';
import { Gamepad2, Monitor, Wrench, Cpu, Check } from 'lucide-react';
import buduLogo from '@/assets/budu-logo.svg';

interface OnboardingViewProps {
  onComplete: () => void;
}

const STEPS = [
  { title: 'Welcome to Budu', description: 'Run your Windows Steam games on Apple Silicon Macs. This wizard will help you get started.', Icon: Gamepad2 },
  { title: 'Install Wine', description: 'Wine translates Windows API calls to macOS. Install via Homebrew: brew install wine-stable', Icon: Monitor },
  { title: 'Set Up Steam', description: 'Budu uses SteamCMD to download your games. Click Setup SteamCMD on the Library page.', Icon: Wrench },
  { title: 'Graphics Translation', description: 'Install DXVK for DirectX 11 to Vulkan to Metal translation. Essential for modern games.', Icon: Cpu },
  { title: "You're Ready", description: 'Download games by App ID, then click Play or use Run .exe to launch any Windows executable.', Icon: Check },
];

export function OnboardingView({ onComplete }: OnboardingViewProps) {
  const [step, setStep] = useState(0);
  const current = STEPS[step];
  if (!current) return null;
  const StepIcon = current.Icon;

  return (
    <div className="flex h-full items-center justify-center">
      <Card className="w-full max-w-lg text-center">
        <CardContent className="flex flex-col items-center gap-6 px-10 py-9">
          {step === 0 ? (
            <img
              src={buduLogo}
              alt="Budu"
              className="h-auto w-full max-w-[320px]"
            />
          ) : (
            <div className="flex h-16 w-16 items-center justify-center rounded-[3px] border border-[#2e5f8d] bg-[#203d59] text-[#8bc7ff]">
              <StepIcon size={30} strokeWidth={1.6} />
            </div>
          )}
          <div>
            <h2 className="text-[22px] font-semibold tracking-[-0.025em] text-white">{current.title}</h2>
            <p className="mx-auto mt-2 max-w-sm text-[13px] leading-relaxed text-white/45">{current.description}</p>
          </div>

          <div className="flex gap-1.5">
            {STEPS.map((_, i) => (
              <div key={i} className={`h-1.5 rounded-[1px] transition-all ${i === step ? 'w-5 bg-[#0a84ff]' : 'w-1.5 bg-[#48484d]'}`} />
            ))}
          </div>

          <div className="mt-1 flex gap-2">
            {step > 0 && <Button variant="ghost" size="sm" onClick={() => setStep(step - 1)}>Back</Button>}
            {step < STEPS.length - 1 ? (
              <Button onClick={() => setStep(step + 1)}>Next</Button>
            ) : (
              <Button onClick={onComplete}>Get Started</Button>
            )}
          </div>
        </CardContent>
      </Card>
    </div>
  );
}
