import { useState } from 'react';
import { Button } from '@/components/ui/Button';
import { Card, CardContent } from '@/components/ui/Card';
import { Gamepad2, Monitor, Wrench, Cpu, Check } from 'lucide-react';

interface OnboardingViewProps {
  onComplete: () => void;
}

const STEPS = [
  { title: 'Welcome to GameRunner', description: 'Run your Windows Steam games on Apple Silicon Macs. This wizard will help you get started.', Icon: Gamepad2 },
  { title: 'Install Wine', description: 'Wine translates Windows API calls to macOS. Install via Homebrew: brew install wine-stable', Icon: Monitor },
  { title: 'Set Up Steam', description: 'GameRunner uses SteamCMD to download your games. Click Setup SteamCMD on the Library page.', Icon: Wrench },
  { title: 'Graphics Translation', description: 'Install DXVK for DirectX 11 to Vulkan to Metal translation. Essential for modern games.', Icon: Cpu },
  { title: "You're Ready", description: 'Download games by App ID, then click Play or use Run .exe to launch any Windows executable.', Icon: Check },
];

export function OnboardingView({ onComplete }: OnboardingViewProps) {
  const [step, setStep] = useState(0);
  const current = STEPS[step];
  if (!current) return null;
  const StepIcon = current.Icon;

  return (
    <div className="flex items-center justify-center h-full">
      <Card className="w-full max-w-lg text-center p-6">
        <CardContent className="flex flex-col items-center gap-5">
          <div className="text-[#7c9c2e]">
            <StepIcon size={40} strokeWidth={1.5} />
          </div>
          <div>
            <h2 className="text-[13px] font-bold uppercase tracking-wider text-[#e0e0d0] text-shadow">{current.title}</h2>
            <p className="text-[11px] text-[#a0a090] mt-2 max-w-sm mx-auto leading-relaxed">{current.description}</p>
          </div>

          <div className="flex gap-1.5">
            {STEPS.map((_, i) => (
              <div key={i} className={`w-2 h-2 border-2 ${i === step ? 'bg-[#7c9c2e] border-[#9dc844]' : 'bg-[#3a3a3a] border-[#4a4a4a]'}`} />
            ))}
          </div>

          <div className="flex gap-2 mt-2">
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
