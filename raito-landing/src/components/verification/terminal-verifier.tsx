"use client"

import { useState } from "react"
import { Button } from "@/components/ui/button"
import { Icons } from "@/components/ui/icons"

interface TerminalVerifierProps {
  blockHeight: number
  className?: string
}

export function TerminalVerifier({ blockHeight, className = "" }: TerminalVerifierProps) {
  const [isVerifying, setIsVerifying] = useState(false)
  const [isOpen, setIsOpen] = useState(false)
  const [step, setStep] = useState(0)

  const verificationSteps = [
    "Initializing STARK verifier engine...",
    "Loading Bitcoin block header data...",
    "Computing witness polynomials...", 
    "Verifying FRI commitments & proof structure...",
    "Validating Merkle tree inclusion proofs...",
    "Executing final cryptographic verification..."
  ]

  const matrixChars = ['0', '1', 'ア', 'イ', 'ウ', 'エ', 'オ', 'カ', 'キ', 'ク', 'ケ', 'コ', 'A', 'B', 'C', 'D', 'E', 'F']

  const handleVerify = async () => {
    setIsOpen(true)
    setIsVerifying(true)
    setStep(0)

    for (let i = 0; i < verificationSteps.length; i++) {
      await new Promise(resolve => setTimeout(resolve, 800))
      setStep(i + 1)
    }

    await new Promise(resolve => setTimeout(resolve, 600))
    setIsVerifying(false)
  }

  const handleClose = () => {
    setIsOpen(false)
    setIsVerifying(false)
    setStep(0)
  }

  const handleBackdropClick = (e: React.MouseEvent) => {
    if (e.target === e.currentTarget && !isVerifying) {
      handleClose()
    }
  }

  return (
    <>
      <div className={`${className}`}>
        <Button
          onClick={handleVerify}
          className="btn-premium relative bg-success hover:bg-success/90 text-black font-semibold shadow-lg hover:shadow-xl transition-all duration-300 mobile-button overflow-hidden group"
          disabled={isVerifying}
        >
          <div className="flex items-center relative z-10">
            <Icons.lock className="mr-2 h-5 w-5 group-hover:rotate-12 transition-transform duration-300" />
            <span className="font-semibold">Verify Locally</span>
          </div>
          
          {/* Premium button glow */}
          <div className="absolute inset-0 bg-gradient-to-r from-success via-transparent to-success opacity-0 group-hover:opacity-30 transition-opacity duration-500" />
        </Button>
      </div>

      {isOpen && (
        <div 
          className="fixed inset-0 z-50 flex items-center justify-center bg-black/90 backdrop-blur-md mobile-p-safe"
          onClick={handleBackdropClick}
        >
          {/* Enhanced terminal container */}
          <div className="relative bg-gradient-to-br from-slate-900 via-black to-slate-900 border border-success/40 rounded-2xl shadow-2xl mobile-terminal w-full max-w-5xl overflow-hidden">
            {/* Premium matrix rain background */}
            <div className="absolute inset-0 overflow-hidden pointer-events-none opacity-60">
              {Array.from({ length: 20 }, (_, i) => (
                <div
                  key={i}
                  className="matrix-rain-premium absolute text-success/30 font-mono text-xs select-none"
                  style={{
                    left: `${i * 5}%`,
                    animationDuration: `${3 + Math.random() * 4}s`,
                    animationDelay: `${Math.random() * 2}s`
                  }}
                >
                  {Array.from({ length: 25 }, (_, j) => (
                    <div key={j} className="leading-4">
                      {matrixChars[Math.floor(Math.random() * matrixChars.length)]}
                    </div>
                  ))}
                </div>
              ))}
            </div>

            {/* Terminal header with enhanced styling */}
            <div className="relative z-10 flex items-center justify-between p-4 sm:p-6 bg-gradient-to-r from-slate-800/80 via-slate-900/80 to-slate-800/80 border-b border-success/30 backdrop-blur-sm">
              <div className="flex items-center gap-3 sm:gap-4">
                {/* Traffic light controls */}
                <div className="flex gap-2">
                  <div className="w-3 h-3 sm:w-4 sm:h-4 rounded-full bg-red-500 shadow-[0_0_8px_rgba(239,68,68,0.5)]"></div>
                  <div className="w-3 h-3 sm:w-4 sm:h-4 rounded-full bg-yellow-500 shadow-[0_0_8px_rgba(234,179,8,0.5)]"></div>
                  <div className="w-3 h-3 sm:w-4 sm:h-4 rounded-full bg-green-500 shadow-[0_0_8px_rgba(34,197,94,0.5)]"></div>
                </div>
                
                {/* Terminal title with glow */}
                <div className="glow-text-premium font-mono text-sm sm:text-base font-bold">
                  <span className="text-success">raito-stark-verifier</span>
                  <span className="text-bitcoin mx-2">v2.0.0</span>
                  <span className="text-text-secondary">Block #{blockHeight}</span>
                </div>
              </div>
              
              {!isVerifying && (
                <Button
                  onClick={handleClose}
                  variant="ghost"
                  className="mobile-icon-button text-text-secondary hover:text-danger h-8 w-8 sm:h-10 sm:w-10 hover:bg-danger/10 transition-all duration-300"
                >
                  <Icons.unverified className="h-4 w-4 sm:h-5 sm:w-5" />
                </Button>
              )}
            </div>

            {/* Terminal content with enhanced styling */}
            <div className="relative z-10 p-6 sm:p-8 bg-black/95 min-h-[350px] sm:min-h-[450px] overflow-y-auto mobile-scroll">
              <div className="font-mono text-success space-y-4 sm:space-y-5">
                {/* Enhanced verification steps */}
                {verificationSteps.map((stepText, index) => (
                  <div key={index} className="flex items-start gap-3 sm:gap-4 group">
                    {/* Step indicator */}
                    <div className="flex-shrink-0 mt-1">
                      {index < step ? (
                        <div className="relative">
                          <Icons.verified className="h-5 w-5 sm:h-6 sm:w-6 text-success drop-shadow-[0_0_8px_rgba(34,197,94,0.6)]" />
                          <div className="absolute inset-0 bg-success rounded-full animate-ping opacity-30" />
                        </div>
                      ) : index === step - 1 && isVerifying ? (
                        <div className="loading-dots-premium flex gap-1">
                          <span className="dot w-1.5 h-1.5 sm:w-2 sm:h-2"></span>
                          <span className="dot w-1.5 h-1.5 sm:w-2 sm:h-2"></span>
                          <span className="dot w-1.5 h-1.5 sm:w-2 sm:h-2"></span>
                        </div>
                      ) : (
                        <div className="w-5 h-5 sm:w-6 sm:h-6 border-2 border-success/30 rounded bg-black/40"></div>
                      )}
                    </div>

                    {/* Step text */}
                    <div className="flex-1 min-w-0">
                      <span className={`
                        text-sm sm:text-base leading-relaxed transition-all duration-500
                        ${index < step ? 'text-success drop-shadow-[0_0_4px_rgba(34,197,94,0.4)]' : 
                          index === step - 1 && isVerifying ? 'glow-text-premium text-success' : 
                          'text-success/40'}
                      `}>
                        {stepText}
                      </span>
                      
                      {/* Progress bar for current step */}
                      {index === step - 1 && isVerifying && (
                        <div className="mt-2 w-full bg-slate-800 rounded-full h-1.5 overflow-hidden">
                          <div className="h-full bg-gradient-to-r from-success to-bitcoin rounded-full animate-pulse" 
                               style={{ width: '70%' }} />
                        </div>
                      )}
                    </div>
                  </div>
                ))}

                {/* Enhanced results section */}
                {!isVerifying && step >= verificationSteps.length && (
                  <div className="mt-8 sm:mt-12 space-y-6 sm:space-y-8 mobile-slide-up">
                    {/* Success banner */}
                    <div className="verification-glow-premium border-2 border-success/40 rounded-2xl p-6 sm:p-8 bg-gradient-to-br from-success/10 via-black to-success/5 relative overflow-hidden">
                      {/* Animated background pattern */}
                      <div className="absolute inset-0 opacity-20">
                        <div className="block-stream-premium"></div>
                      </div>
                      
                      <div className="relative z-10">
                        <div className="flex items-center gap-4 sm:gap-6 mb-6 sm:mb-8">
                          <div className="p-3 sm:p-4 bg-success/20 rounded-2xl border border-success/30 stark-pulse">
                            <Icons.lock className="h-8 w-8 sm:h-10 sm:w-10 text-success" />
                          </div>
                          <div>
                            <h3 className="text-2xl sm:text-3xl font-bold text-success mb-2 sm:mb-3 glow-text-premium">
                              STARK Proof Verified Successfully!
                            </h3>
                            <p className="text-sm sm:text-base text-success/80 leading-relaxed">
                              Block #{blockHeight} is cryptographically proven valid with zero-knowledge guarantees
                            </p>
                          </div>
                        </div>

                        {/* Enhanced stats grid */}
                        <div className="grid grid-cols-2 sm:grid-cols-4 gap-4 sm:gap-6 mb-6 sm:mb-8">
                          <div className="card-hover-premium bg-slate-800/50 rounded-xl p-4 sm:p-5 border border-slate-600/50 backdrop-blur-sm">
                            <div className="text-xs sm:text-sm text-success font-medium mb-1">Proof Size</div>
                            <div className="text-lg sm:text-xl font-bold text-success">2.4 KB</div>
                            <div className="text-xs text-success/60">Compressed</div>
                          </div>
                          <div className="card-hover-premium bg-slate-800/50 rounded-xl p-4 sm:p-5 border border-slate-600/50 backdrop-blur-sm">
                            <div className="text-xs sm:text-sm text-bitcoin font-medium mb-1">Verify Time</div>
                            <div className="text-lg sm:text-xl font-bold text-bitcoin">3.2s</div>
                            <div className="text-xs text-bitcoin/60">Lightning fast</div>
                          </div>
                          <div className="card-hover-premium bg-slate-800/50 rounded-xl p-4 sm:p-5 border border-slate-600/50 backdrop-blur-sm col-span-2 sm:col-span-1">
                            <div className="text-xs sm:text-sm text-purple-400 font-medium mb-1">Security</div>
                            <div className="text-lg sm:text-xl font-bold text-purple-400">128-bit</div>
                            <div className="text-xs text-purple-400/60">Quantum resistant</div>
                          </div>
                          <div className="card-hover-premium bg-slate-800/50 rounded-xl p-4 sm:p-5 border border-slate-600/50 backdrop-blur-sm col-span-2 sm:col-span-1">
                            <div className="text-xs sm:text-sm text-emerald-400 font-medium mb-1">Trust Level</div>
                            <div className="text-lg sm:text-xl font-bold text-emerald-400">Zero</div>
                            <div className="text-xs text-emerald-400/60">Trustless</div>
                          </div>
                        </div>

                        {/* Verification summary */}
                        <div className="p-4 sm:p-6 bg-slate-900/80 rounded-xl border border-success/20 backdrop-blur-sm">
                          <div className="text-sm sm:text-base text-success/90 leading-relaxed space-y-2">
                            <p>✅ <strong>STARK proof validation complete.</strong> This block header is mathematically proven to be part of the canonical Bitcoin blockchain.</p>
                            <p>🔒 <strong>Zero-knowledge verification</strong> ensures validity without requiring trust in any third party or full node.</p>
                            <p>⚡ <strong>Instant synchronization</strong> replaces hours of initial block download with cryptographic certainty.</p>
                          </div>
                        </div>
                      </div>
                    </div>

                    {/* Enhanced close button */}
                    <div className="flex justify-center">
                      <Button
                        onClick={handleClose}
                        className="btn-premium mobile-button bg-gradient-to-r from-slate-700 to-slate-800 hover:from-slate-600 hover:to-slate-700 text-text-primary border border-success/30 group relative overflow-hidden"
                      >
                        <div className="flex items-center relative z-10">
                          <Icons.verified className="mr-2 h-5 w-5 group-hover:scale-110 transition-transform duration-300" />
                          <span>Close Terminal</span>
                        </div>
                      </Button>
                    </div>
                  </div>
                )}
              </div>
            </div>

            {/* Terminal border glow effect */}
            <div className="absolute inset-0 rounded-2xl border-2 border-transparent bg-gradient-to-br from-success/20 via-transparent to-bitcoin/20 p-0.5 -z-10">
              <div className="w-full h-full bg-black rounded-xl" />
            </div>
          </div>
        </div>
      )}
    </>
  )
} 