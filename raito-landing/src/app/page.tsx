"use client"

import { useState, useEffect } from "react"
import Link from "next/link"
import Image from "next/image"
import { Button } from "@/components/ui/button"
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card"
import { Icons } from "@/components/ui/icons"
import { HashFlicker } from "@/components/ui/hash-flicker"
import { MobileHashInput } from "@/components/ui/mobile-hash-input"
import { createRaitoSpvSdk } from '@starkware-bitcoin/spv-verify'

type CheckState = 'idle' | 'checking' | 'found' | 'not-found' | 'invalid'

// Interface for sample transactions (currently unused but kept for future features)
// interface SampleTx {
//   id: string
//   hash: string
//   description: string
//   type: 'valid' | 'invalid'
//   expectedResult?: string
// }

// Sample transactions for testing (currently unused but kept for future features)
// const sampleTransactions: SampleTx[] = [
//   {
//     id: 'valid-1',
//     hash: '4f1b987645e596329b985064b1ce33046e4e293a08fd961193c8ddbb1ca219cc',
//     description: 'Real Bitcoin transaction (from SDK docs)',
//     type: 'valid',
//     expectedResult: 'Verified with STARK proof'
//   },
//   {
//     id: 'valid-2', 
//     hash: 'd4e5f67890123456789012345678901234567890123456789012345678abcdef',
//     description: 'Sample transaction in Block #869999',
//     type: 'valid',
//     expectedResult: 'Found in Block #869999'
//   },
//   {
//     id: 'valid-3',
//     hash: '7890123456789012345678901234567890123456789012345678901234abcdef',
//     description: 'Sample transaction in Block #869998',
//     type: 'valid', 
//     expectedResult: 'Found in Block #869998'
//   },
//   {
//     id: 'invalid-1',
//     hash: 'ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff',
//     description: 'Non-existent transaction (all F\'s)',
//     type: 'invalid',
//     expectedResult: 'Not found in blockchain'
//   },
//   {
//     id: 'invalid-2',
//     hash: '0000000000000000000000000000000000000000000000000000000000000000',
//     description: 'Invalid transaction (all zeros)',
//     type: 'invalid',
//     expectedResult: 'Not found in blockchain'
//   },
//   {
//     id: 'invalid-3',
//     hash: 'deadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeef',
//     description: 'Test transaction (deadbeef pattern)',
//     type: 'invalid',
//     expectedResult: 'Not found in blockchain'
//   }
// ]

export default function HomePage() {
  const [txid, setTxid] = useState('4f1b987645e596329b985064b1ce33046e4e293a08fd961193c8ddbb1ca219cc')
  const [state, setState] = useState<CheckState>('idle')
  const [globalRecentHeight, setGlobalRecentHeight] = useState<number | null>(null)
  const [result, setResult] = useState<{ 
    blockHeight?: number; 
    txid?: string; 
    proof?: unknown;
    blockHash?: string;
    recentHeight?: number;
  } | null>(null)
  
  // Fetch recent proven height on component mount
  useEffect(() => {
    const fetchRecentHeight = async () => {
      try {
        const sdk = createRaitoSpvSdk({verifierConfig: {min_work: '0'}})
        await sdk.init()
        const recentHeight = await sdk.fetchRecentProvenHeight()
        setGlobalRecentHeight(recentHeight)
      } catch (error) {
        console.error('Failed to fetch recent height:', error)
      }
    }
    
    fetchRecentHeight()
  }, [])
  
  const validateTxid = (value: string): boolean => {
    return /^[a-fA-F0-9]{64}$/.test(value)
  }
  
  const handleCheck = async () => {
    if (!validateTxid(txid)) {
      setState('invalid')
      return
    }
    
    setState('checking')
    
    try {
      // Create SDK instance
      const sdk = createRaitoSpvSdk({verifierConfig: {min_work: '0'}})
      
      // Initialize the SDK (loads WASM module)
      await sdk.init()
      
      // Fetch recent proven height
      const recentHeight = await sdk.fetchRecentProvenHeight()
      
      // Verify the transaction using the new API
      const transaction = await sdk.verifyTransaction(txid)
      
      if (transaction) {
        setState('found')
        setResult({ 
          blockHeight: recentHeight, // The transaction is verified to be in a recent block
          txid, 
          proof: transaction, // Store the transaction data as proof
          blockHash: '', // Block hash not directly available from transaction verification
          recentHeight
        })
      } else {
        setState('not-found')
        setResult({ txid })
      }
    } catch (error) {
      console.error('Verification error:', error)
      setState('not-found')
      setResult({ txid })
    }
  }
  
  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault()
    handleCheck()
  }
  
  // Sample click handler (currently unused but kept for future features)
  // const handleSampleClick = (hash: string) => {
  //   setTxid(hash)
  //   setState('idle')
  //   setResult(null)
  // }
  
  const handleNewCheck = () => {
    setState('idle')
    setResult(null)
    setTxid('')
  }

  return (
    <div className="relative page-transition-premium">
      {/* Enhanced block stream background */}
      <div className="absolute inset-0 block-stream-premium opacity-40" />
      
      {/* Hero Section */}
      <section className="relative px-4 py-24 sm:px-6 lg:px-8 overflow-hidden">
        {/* Floating Bitcoin particles */}
        <div className="absolute inset-0 pointer-events-none">
          {[...Array(8)].map((_, i) => (
            <div
              key={i}
              className={`
                absolute w-2 h-2 bg-bitcoin/20 rounded-full
                float-premium opacity-30
              `}
              style={{
                left: `${10 + i * 12}%`,
                top: `${20 + (i % 3) * 25}%`,
                animationDelay: `${i * 0.8}s`,
                animationDuration: `${4 + (i % 3)}s`
              }}
            />
          ))}
        </div>

        <div className="mx-auto max-w-4xl text-center relative z-10">
          <h1 className="mobile-h1 lg:text-7xl font-extrabold tracking-tight text-text-primary mb-8 leading-tight">
            <span className="inline-block hover:scale-105 transition-transform duration-300 cursor-default">
              Don&apos;t trust,
            </span>
            <span className="text-bitcoin block mt-2 hover:drop-shadow-[0_0_20px_rgba(247,147,26,0.6)] transition-all duration-500 cursor-default glow-text-premium">
              verify
            </span>
          </h1>
          
          <p className="mobile-text-lg lg:text-2xl text-text-secondary mb-12 max-w-3xl mx-auto leading-relaxed">
            Bitcoin light client with{" "}
            <span className="text-bitcoin font-semibold relative">
              <span className="relative z-10">STARK proofs</span>
              <span className="absolute inset-0 bg-bitcoin/10 blur-md rounded" />
            </span>
            <br />
            Don&apos;t trust, verify — with zero-knowledge cryptography.
          </p>
          
          <div className="flex flex-col sm:flex-row gap-6 justify-center items-center mb-16">
            <Button 
              size="lg" 
              asChild 
              className="btn-premium group relative overflow-hidden mobile-button min-w-[200px]"
            >
              <Link href="https://github.com/starkware-bitcoin/raito" target="_blank">
                <div className="flex items-center relative z-10">
                  <Icons.github className="mr-2 h-5 w-5 group-hover:rotate-12 transition-transform duration-300" />
                  <span className="font-semibold">View on GitHub</span>
                </div>
              </Link>
            </Button>
            
            <Button 
              variant="outline" 
              size="lg" 
              asChild
              className="card-hover-premium border-bitcoin/30 hover:border-bitcoin text-text-primary hover:text-bitcoin mobile-button min-w-[200px] group"
            >
              <Link href="https://zerosync.org" target="_blank">
                <span>Learn More</span>
                <Icons.externalLink className="ml-2 h-4 w-4 group-hover:translate-x-1 group-hover:-translate-y-1 transition-transform duration-300" />
              </Link>
            </Button>
          </div>

          {/* Enhanced stats ticker */}
          <div className="grid grid-cols-2 sm:grid-cols-5 gap-4 sm:gap-6 text-center">
            <div className="card-hover-premium p-4 rounded-xl bg-surface/50 border border-slate-700/50 backdrop-blur-sm">
              <div className="text-2xl sm:text-3xl font-bold text-blue-400 mb-1">
                {globalRecentHeight ? `${globalRecentHeight.toLocaleString()}` : 'loading...'}
              </div>
              <div className="text-xs sm:text-sm text-text-secondary">Proven Height</div>
            </div>
            <div className="card-hover-premium p-4 rounded-xl bg-surface/50 border border-slate-700/50 backdrop-blur-sm">
              <div className="text-2xl sm:text-3xl font-bold text-bitcoin mb-1">1.2MB</div>
              <div className="text-xs sm:text-sm text-text-secondary">Proof Size</div>
            </div>
            <div className="card-hover-premium p-4 rounded-xl bg-surface/50 border border-slate-700/50 backdrop-blur-sm">
              <div className="text-2xl sm:text-3xl font-bold text-success mb-1">&lt;1s</div>
              <div className="text-xs sm:text-sm text-text-secondary">Verify Time</div>
            </div>
            <div className="card-hover-premium p-4 rounded-xl bg-surface/50 border border-slate-700/50 backdrop-blur-sm">
              <div className="text-2xl sm:text-3xl font-bold text-purple-400 mb-1">96b</div>
              <div className="text-xs sm:text-sm text-text-secondary">Security</div>
            </div>
            <div className="card-hover-premium p-4 rounded-xl bg-surface/50 border border-slate-700/50 backdrop-blur-sm">
              <div className="text-2xl sm:text-3xl font-bold text-emerald-400 mb-1">0</div>
              <div className="text-xs sm:text-sm text-text-secondary">Trust Required</div>
            </div>
          </div>
        </div>
      </section>
      

      {/* Transaction Verification Section */}
      <section id="verify" className="relative px-4 py-20 sm:px-6 lg:px-8 bg-gradient-to-br from-surface via-surface-alt to-surface overflow-hidden">
        <div className="absolute inset-0 bg-bitcoin/5 opacity-30" />
        <div className="relative z-10 mx-auto max-w-7xl">
          <div className="text-center mb-12">
            <div className="flex items-center justify-center gap-2 sm:gap-3 mb-3 sm:mb-4">
              <div className="p-2 sm:p-3 bg-bitcoin/20 rounded-xl border border-bitcoin/30">
                <Icons.search className="h-6 w-6 sm:h-8 sm:w-8 text-bitcoin" />
              </div>
              <h2 className="mobile-h2 font-bold text-text-primary">
                Transaction Verification
              </h2>
            </div>
            <p className="mobile-text-lg text-text-secondary max-w-3xl mx-auto leading-relaxed px-4">
              Verify Bitcoin transaction inclusion using STARK proofs. 
              <span className="text-bitcoin font-semibold"> Don&apos;t trust, verify.</span>
            </p>
          </div>


          <div className="flex justify-center">
            {/* Main Verification Panel */}
            <div className="w-full max-w-6xl space-y-4 sm:space-y-6">
              <Card className="theme-transition hover:shadow-xl hover:border-bitcoin/30 transition-all duration-300">
                <CardHeader className="pb-4 sm:pb-6">
                  <CardTitle className="flex items-center gap-2 sm:gap-3 mobile-h3">
                    <Icons.activity className="h-5 w-5 sm:h-6 sm:w-6 text-bitcoin" />
                    Enter Transaction ID
                  </CardTitle>
                  <p className="text-text-secondary text-sm sm:text-base">
                    Paste a 64-character hexadecimal transaction hash
                  </p>
                </CardHeader>
                <CardContent>
                  <form onSubmit={handleSubmit} className="space-y-4 sm:space-y-6">
                    <MobileHashInput
                      value={txid}
                      onChange={setTxid}

                      disabled={state === 'checking'}
                      isValid={state === 'found'}
                      isInvalid={state === 'invalid'}
                      onPaste={() => {
                        if (state === 'invalid') setState('idle')
                      }}
                    />
                    
                    <Button 
                      type="submit" 
                      className="mobile-button w-full font-semibold" 
                      disabled={state === 'checking' || !txid.trim()}
                    >
                      {state === 'checking' ? (
                        <>
                          <Icons.spinner className="mr-2 sm:mr-3 h-4 w-4 sm:h-5 sm:w-5 animate-spin" />
                          Verifying Transaction...
                        </>
                      ) : (
                        <>
                          <Icons.search className="mr-2 sm:mr-3 h-4 w-4 sm:h-5 sm:w-5" />
                          Verify Transaction Inclusion
                        </>
                      )}
                    </Button>
                  </form>
                </CardContent>
              </Card>

              {/* Results */}
              {state === 'found' && result && (
                <Card className="border-success/50 bg-gradient-to-r from-success/10 to-success/5 mobile-slide-up">
                  <CardContent className="pt-4 sm:pt-6">
                    <div className="flex items-start gap-3 sm:gap-4">
                      <div className="p-2 sm:p-3 bg-success/20 rounded-xl border border-success/30 flex-shrink-0">
                        <Icons.verified className="h-6 w-6 sm:h-8 sm:w-8 text-success" />
                      </div>
                      <div className="flex-1 min-w-0">
                        <h3 className="mobile-h3 font-bold text-success mb-2 sm:mb-3">Transaction Verified!</h3>
                          
                          <p className="text-text-secondary leading-relaxed text-sm sm:text-base">
                          This transaction is confirmed and included in the canonical Bitcoin blockchain. It&apos;s covered by a STARK proof, ensuring mathematical certainty of its inclusion without requiring trust in third parties.
                          </p>
                          
                        <div className="flex flex-col sm:flex-row flex-wrap gap-2 sm:gap-3 mt-4">
                            <Button variant="ghost" onClick={handleNewCheck} className="mobile-button">
                              <Icons.search className="mr-2 h-4 w-4" />
                              Verify Another
                            </Button>
                        </div>
                      </div>
                    </div>
                  </CardContent>
                </Card>
              )}
              
              {state === 'not-found' && result && (
                <Card className="border-danger/50 bg-gradient-to-r from-danger/10 to-danger/5 mobile-slide-up">
                  <CardContent className="pt-4 sm:pt-6">
                    <div className="flex items-start gap-3 sm:gap-4">
                      <div className="p-2 sm:p-3 bg-danger/20 rounded-xl border border-danger/30 flex-shrink-0">
                        <Icons.unverified className="h-6 w-6 sm:h-8 sm:w-8 text-danger" />
                      </div>
                      <div className="flex-1 min-w-0">
                        <h3 className="mobile-h3 font-bold text-danger mb-2 sm:mb-3">Transaction Not Found</h3>
                        
                        <div className="space-y-3 sm:space-y-4">
                          <div className="p-3 sm:p-4 bg-surface-alt rounded-lg border border-slate-600">
                            <label className="text-xs sm:text-sm font-medium text-text-secondary block mb-2">Searched Hash</label>
                            <HashFlicker 
                              hash={result.txid || ''} 
                              className="text-text-primary break-all leading-relaxed text-sm sm:text-base"
                              copyable={true}
                            />
                          </div>
                          
                          <div className="p-3 sm:p-4 bg-danger/10 rounded-lg border border-danger/30">
                            <h4 className="font-semibold text-danger mb-2 text-sm sm:text-base">Possible Reasons:</h4>
                            <ul className="space-y-1 text-xs sm:text-sm text-text-secondary">
                              <li>• Transaction doesn&apos;t exist or hasn&apos;t been mined yet</li>
                              <li>• Hash might be from a different network (testnet, etc.)</li>
                              <li>• Transaction could be in mempool but not yet confirmed</li>
                              <li>• Not included in our current verified dataset</li>
                            </ul>
                          </div>
                          
                          <Button variant="ghost" onClick={handleNewCheck} className="mobile-button w-full sm:w-auto">
                            <Icons.search className="mr-2 h-4 w-4" />
                            Try Another Transaction
                          </Button>
                        </div>
                      </div>
                    </div>
                  </CardContent>
                </Card>
              )}
            </div>

          </div>
        </div>
      </section>
      

      {/* TrustMeBro ZK ready Bitcoin Explorer Section */}
      <section id="trustmebro" className="relative px-4 py-20 sm:px-6 lg:px-8 bg-gradient-to-br from-surface via-surface-alt to-surface overflow-hidden">
        <div className="absolute inset-0 bg-bitcoin/5 opacity-30" />
        <div className="relative z-10 mx-auto max-w-7xl">
          <div className="text-center mb-12">
            <div className="flex items-center justify-center gap-2 sm:gap-3 mb-3 sm:mb-4">
              <div className="p-2 sm:p-3 bg-bitcoin/20 rounded-xl border border-bitcoin/30">
                <Icons.search className="h-6 w-6 sm:h-8 sm:w-8 text-bitcoin" />
              </div>
              <h2 className="mobile-h2 font-bold text-text-primary">
                TrustMeBro ZK ready Bitcoin Explorer
              </h2>
            </div>
            <p className="mobile-text-lg text-text-secondary max-w-3xl mx-auto leading-relaxed px-4">
              A zero-knowledge ready Bitcoin explorer that provides cryptographic proof of transaction inclusion without requiring trust in third parties.
              <span className="text-bitcoin font-semibold"> Don&apos;t trust, verify.</span>
            </p>
          </div>

          <div className="flex justify-center">
            <div className="w-full max-w-6xl">
              <Card className="theme-transition hover:shadow-xl hover:border-bitcoin/30 transition-all duration-300">
                <CardContent className="p-6 sm:p-8">
                  <div className="space-y-6">
                    <div className="relative">
                      <Image 
                        src="/trustmebro-screenshot.png" 
                        alt="TrustMeBro ZK ready Bitcoin Explorer Screenshot" 
                        width={1200}
                        height={800}
                        className="w-full h-auto rounded-lg border border-slate-700/50 shadow-lg"
                      />
                      <div className="absolute inset-0 bg-gradient-to-t from-black/20 to-transparent rounded-lg" />
                    </div>
                    
                    <div className="text-center">
                      <Button 
                        size="lg" 
                        asChild 
                        className="btn-premium group relative overflow-hidden mobile-button min-w-[200px]"
                      >
                        <Link href="https://trustmebro.starkwarebitcoin.dev/" target="_blank">
                          <div className="flex items-center relative z-10">
                            <Icons.externalLink className="mr-2 h-5 w-5 group-hover:rotate-12 transition-transform duration-300" />
                            <span className="font-semibold">Explore TrustMeBro</span>
                          </div>
                        </Link>
                      </Button>
                    </div>
                  </div>
                </CardContent>
              </Card>
            </div>
          </div>
        </div>
      </section>
      

      {/* Developer Resources Section */}
      <section id="developers" className="relative px-4 py-20 sm:px-6 lg:px-8">
        <div className="mx-auto max-w-6xl">
          <div className="mb-12 text-center">
            <h2 className="text-4xl font-bold text-text-primary mb-4">
              Developer Resources
            </h2>
            <p className="text-text-secondary text-lg max-w-3xl mx-auto">
              Build on top of Raito&apos;s Bitcoin STARK verification infrastructure. 
              Access APIs, SDKs, and documentation to integrate trustless verification into your applications.
            </p>
          </div>
          
          
          <div className="grid grid-cols-1 gap-8 mb-12">
            {/* JavaScript SDK */}
            <Card>
              <CardHeader>
                <CardTitle className="flex items-center gap-2">
                  <Icons.download className="h-5 w-5 text-bitcoin" />
                  JavaScript SDK
                </CardTitle>
              </CardHeader>
              <CardContent className="space-y-4">
                <p className="text-text-secondary text-sm">
                  Integrate Raito verification into your web applications
                </p>
                
                <div className="p-4 bg-bg-base rounded-lg border border-slate-700">
                  <pre className="font-mono text-sm text-text-primary overflow-x-auto">
                    <code>{`npm install @starkware-bitcoin/spv-verify

import { createRaitoSpvSdk } from '@starkware-bitcoin/spv-verify'

async function verifyTransaction() {
  // Create SDK instance
  const sdk = createRaitoSpvSdk({verifierConfig: {min_work: '0'}})
  
  // Initialize the SDK (loads WASM module)
  await sdk.init()
  
  // Fetch recent proven height
  const recentHeight = await sdk.fetchRecentProvenHeight()
  console.log('Most recent proven block height:', recentHeight)
  
  // Verify a transaction
  const txid = '4f1b987645e596329b985064b1ce33046e4e293a08fd961193c8ddbb1ca219cc'
  
  // Verify the transaction using the new API
  const transaction = await sdk.verifyTransaction(txid)

  console.log('Verification result:', transaction ? 'Valid' : 'Invalid')
  if (transaction) {
    console.log('Transaction details:', transaction)
  }
}`}</code>
                  </pre>
                </div>
                
                <Button variant="outline" className="w-full" asChild>
                  <Link href="https://github.com/starkware-bitcoin/raito/tree/main/raito-spv-verify-sdk" target="_blank">
                    <Icons.externalLink className="mr-2 h-4 w-4" />
                    View on GitHub
                  </Link>
                </Button>
              </CardContent>
            </Card>
            
            {/* CLI Tools */}
            <Card>
              <CardHeader>
                <CardTitle className="flex items-center gap-2">
                  <Icons.search className="h-5 w-5 text-bitcoin" />
                  Command Line Tools
                </CardTitle>
              </CardHeader>
              <CardContent className="space-y-4">
                <p className="text-text-secondary text-sm">
                  Verify proofs and interact with Bitcoin data from the terminal
                </p>
                
                <div className="p-4 bg-bg-base rounded-lg border border-slate-700">
                  <pre className="font-mono text-sm text-text-primary overflow-x-auto">
                    <code>
                      {`raito-spv-client fetch --txid <hex_txid> --proof-path tx_proof.bin.bz2
raito-spv-client verify --proof-path ./proofs/tx_proof.bin.bz2`}
                    </code>
                  </pre>
                </div>
                
                <Button variant="outline" className="w-full" asChild>
                  <Link href="https://github.com/starkware-bitcoin/raito/tree/main/crates/raito-spv-client" target="_blank">
                    <Icons.externalLink className="mr-2 h-4 w-4" />
                    View on GitHub
                  </Link>
                </Button>
              </CardContent>
            </Card>
            
            {/* Node Integration */}
            <Card>
              <CardHeader>
                <CardTitle className="flex items-center gap-2">
                  <Icons.verified className="h-5 w-5 text-bitcoin" />
                  Bitcoin Node Plugin
                </CardTitle>
              </CardHeader>
              <CardContent className="space-y-4">
                <p className="text-text-secondary text-sm">
                  Add STARK verification capabilities to your Bitcoin node (In the works...).
                </p>
                
                <div className="space-y-3">
                  <div className="flex items-center gap-3 p-3 bg-surface-alt rounded-lg">
                    <Icons.lightning className="h-5 w-5 text-bitcoin" />
                    <div>
                      <h4 className="font-medium text-text-primary">Fast IBD</h4>
                      <p className="text-xs text-text-secondary">Initial Block Download in minutes</p>
                    </div>
                  </div>
                  
                  <div className="flex items-center gap-3 p-3 bg-surface-alt rounded-lg">
                    <Icons.lock className="h-5 w-5 text-success" />
                    <div>
                      <h4 className="font-medium text-text-primary">Trustless Sync</h4>
                      <p className="text-xs text-text-secondary">Verify without trusting peers</p>
                    </div>
                  </div>
                  
                  <div className="flex items-center gap-3 p-3 bg-surface-alt rounded-lg">
                    <Icons.verified className="h-5 w-5 text-bitcoin" />
                    <div>
                      <h4 className="font-medium text-text-primary">STARK Proofs</h4>
                      <p className="text-xs text-text-secondary">Cryptographic verification</p>
                    </div>
                  </div>
                </div>
              </CardContent>
            </Card>
          </div>
          
        </div>
      </section>
    </div>
  )
}
