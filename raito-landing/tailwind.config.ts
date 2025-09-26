import type { Config } from "tailwindcss";

const config: Config = {
  content: [
    "./src/pages/**/*.{js,ts,jsx,tsx,mdx}",
    "./src/components/**/*.{js,ts,jsx,tsx,mdx}",
    "./src/app/**/*.{js,ts,jsx,tsx,mdx}",
  ],
  theme: {
    extend: {
      colors: {
        border: "hsl(var(--border))",
        input: "hsl(var(--input))",
        ring: "hsl(var(--ring))",
        background: "hsl(var(--background))",
        foreground: "hsl(var(--foreground))",
        primary: {
          DEFAULT: "hsl(var(--primary))",
          foreground: "hsl(var(--primary-foreground))",
        },
        secondary: {
          DEFAULT: "hsl(var(--secondary))",
          foreground: "hsl(var(--secondary-foreground))",
        },
        destructive: {
          DEFAULT: "hsl(var(--destructive))",
          foreground: "hsl(var(--destructive-foreground))",
        },
        muted: {
          DEFAULT: "hsl(var(--muted))",
          foreground: "hsl(var(--muted-foreground))",
        },
        accent: {
          DEFAULT: "hsl(var(--accent))",
          foreground: "hsl(var(--accent-foreground))",
        },
        card: {
          DEFAULT: "hsl(var(--card))",
          foreground: "hsl(var(--card-foreground))",
        },
        bitcoin: "#F7931A",
        "bitcoin-dark": "#d67d15",
        "bg-base": "#0D0D0D",
        surface: "#131313",
        "surface-alt": "#1C1C1C",
        "text-primary": "#FFFFFF",
        "text-secondary": "#C7C7C7",
        success: "#00D26A",
        danger: "#FF5470",
      },
      fontFamily: {
        sans: ["Inter", "sans-serif"],
        mono: ['"JetBrains Mono"', "monospace"],
      },
      boxShadow: {
        card: "0 2px 6px 0 rgba(0,0,0,0.55)",
        glow: "0 0 16px 0 rgba(247,147,26,0.6)",
      },
      keyframes: {
        "block-stream": {
          "0%": { transform: "translateX(0)" },
          "100%": { transform: "translateX(-50%)" },
        },
        "hash-flicker": {
          "0%, 100%": { opacity: "1" },
          "50%": { opacity: "0.7" },
        },
        "lock-in": {
          "0%": { transform: "scale(0.8)", opacity: "0.5" },
          "100%": { transform: "scale(1)", opacity: "1" },
        },
      },
      animation: {
        "block-stream": "block-stream 45s linear infinite",
        "hash-flicker": "hash-flicker 0.6s ease-in-out",
        "lock-in": "lock-in 0.4s ease-out",
      },
    },
  },
  plugins: [],
};

export default config;
