import type { Metadata, Viewport } from "next";
import { Inter, JetBrains_Mono } from "next/font/google";
import "./globals.css";
import Navigation from "@/components/layout/navigation";
import Footer from "@/components/layout/footer";
import { ThemeProvider } from "@/components/theme/theme-provider";
import { Analytics } from '@vercel/analytics/react';
import { SpeedInsights } from '@vercel/speed-insights/next';

const inter = Inter({
  subsets: ["latin"],
  variable: "--font-inter",
  display: 'swap',
});

const jetbrainsMono = JetBrains_Mono({
  subsets: ["latin"],
  variable: "--font-jetbrains-mono", 
  display: 'swap',
});

const siteUrl = 'https://raito.wtf';
const siteName = 'Raito';
const siteTitle = 'Raito - Bitcoin STARK Verification Portal';
const siteDescription = "Don't trust, verify — with STARK proofs. Explore Bitcoin blocks with instant, trustless verification using zero-knowledge proofs. Blazing fast light client verification for Bitcoin.";

export const metadata: Metadata = {
  metadataBase: new URL(siteUrl),
  title: {
    default: siteTitle,
    template: `%s | ${siteName}`,
  },
  description: siteDescription,
  keywords: [
    "Bitcoin", 
    "STARK", 
    "zero-knowledge", 
    "proof", 
    "verification", 
    "blockchain", 
    "trustless",
    "light client",
    "block explorer",
    "transaction verification",
    "cryptographic proof",
    "Cairo",
    "StarkNet",
    "ZeroSync",
    "initial block download",
    "IBD",
    "consensus verification"
  ],
  authors: [{ name: "Raito Team", url: "https://github.com/keep-starknet-strange" }],
  creator: "Raito Team",
  publisher: "Raito Team", 
  applicationName: siteName,
  generator: "Next.js",
  referrer: "origin-when-cross-origin",
  robots: {
    index: true,
    follow: true,
    nocache: true,
    googleBot: {
      index: true,
      follow: true,
      noimageindex: false,
      "max-video-preview": -1,
      "max-image-preview": "large",
      "max-snippet": -1,
    },
  },
  alternates: {
    canonical: siteUrl,
  },
  icons: {
    icon: [
      { url: '/favicon.ico', sizes: 'any' },
      { url: '/favicon.svg', type: 'image/svg+xml' },
      { url: '/favicon-16x16.png', sizes: '16x16', type: 'image/png' },
      { url: '/favicon-32x32.png', sizes: '32x32', type: 'image/png' },
      { url: '/favicon-96x96.png', sizes: '96x96', type: 'image/png' },
    ],
    apple: [
      { url: '/apple-touch-icon.png', sizes: '180x180', type: 'image/png' },
    ],
    other: [
      {
        rel: 'icon',
        url: '/bitcoin-logo.svg',
        type: 'image/svg+xml',
      },
    ],
  },
  manifest: '/site.webmanifest',
  openGraph: {
    type: "website",
    locale: "en_US",
    url: siteUrl,
    title: siteTitle,
    description: siteDescription,
    siteName: siteName,
    images: [
      {
        url: `${siteUrl}/bitcoin-logo.png`,
        width: 1200,
        height: 630,
        alt: `${siteName} - Bitcoin STARK Verification`,
      },
      {
        url: `${siteUrl}/android-chrome-512x512.png`,
        width: 512,
        height: 512,
        alt: `${siteName} Logo`,
      },
    ],
  },
  twitter: {
    card: "summary_large_image",
    title: siteTitle,
    description: siteDescription,
    creator: "@RaitoTeam",
    images: [`${siteUrl}/bitcoin-logo.png`],
  },
  category: 'technology',
  classification: 'Bitcoin Tools',
  other: {
    'mobile-web-app-capable': 'yes',
    'apple-mobile-web-app-capable': 'yes',
    'apple-mobile-web-app-status-bar-style': 'black-translucent',
    'format-detection': 'telephone=no',
    'msapplication-TileColor': '#F7931A',
    'msapplication-config': 'none',
  },
};

export const viewport: Viewport = {
  themeColor: [
    { media: "(prefers-color-scheme: light)", color: "#FFFFFF" },
    { media: "(prefers-color-scheme: dark)", color: "#0D0D0D" },
  ],
  width: "device-width",
  initialScale: 1,
  maximumScale: 5,
  userScalable: true,
  viewportFit: 'cover',
};

// Structured Data for SEO
const structuredData = {
  '@context': 'https://schema.org',
  '@type': 'WebApplication',
  name: siteName,
  description: siteDescription,
  url: siteUrl,
  applicationCategory: 'FinanceApplication',
  operatingSystem: 'Any',
  offers: {
    '@type': 'Offer',
    price: '0',
    priceCurrency: 'USD',
  },
  author: {
    '@type': 'Organization',
    name: 'Raito Team',
    url: 'https://github.com/keep-starknet-strange',
  },
  publisher: {
    '@type': 'Organization',
    name: 'Raito Team',
  },
  about: {
    '@type': 'Thing',
    name: 'Bitcoin',
    description: 'Cryptocurrency and blockchain technology',
  },
  keywords: 'Bitcoin, STARK, zero-knowledge proof, verification, blockchain, trustless',
  mainEntity: {
    '@type': 'SoftwareApplication',
    name: 'Bitcoin STARK Verifier',
    description: 'Trustless Bitcoin block and transaction verification using STARK proofs',
  },
};

export default function RootLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return (
    <html lang="en" suppressHydrationWarning>
      <head>
        {/* Structured Data */}
        <script
          type="application/ld+json"
          dangerouslySetInnerHTML={{ __html: JSON.stringify(structuredData) }}
        />
        
        {/* Preconnect to external domains */}
        <link rel="preconnect" href="https://fonts.googleapis.com" />
        <link rel="preconnect" href="https://fonts.gstatic.com" crossOrigin="anonymous" />
        <link rel="preconnect" href="https://vitals.vercel-analytics.com" />
        
        {/* DNS Prefetch */}
        <link rel="dns-prefetch" href="//fonts.googleapis.com" />
        <link rel="dns-prefetch" href="//fonts.gstatic.com" />
        
        {/* Security Headers */}
        <meta name="referrer" content="origin-when-cross-origin" />
        
        {/* Additional SEO Meta Tags */}
        <meta name="google-site-verification" content="" />
        <meta name="msvalidate.01" content="" />
        <meta name="yandex-verification" content="" />
        
        {/* Cache Control */}
        <meta httpEquiv="Cache-Control" content="public, max-age=31536000, immutable" />
      </head>
      <body
        className={`${inter.variable} ${jetbrainsMono.variable} font-sans antialiased`}
      >
        <ThemeProvider defaultTheme="dark" storageKey="raito-theme">
          <div className="min-h-screen bg-background text-foreground theme-transition">
            <Navigation />
            <main className="min-h-screen">
              {children}
            </main>
            <Footer />
          </div>
        </ThemeProvider>
        
        {/* Analytics */}
        <Analytics />
        <SpeedInsights />
      </body>
    </html>
  );
}
