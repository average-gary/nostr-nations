/** @type {import('tailwindcss').Config} */
export default {
  content: ['./index.html', './src/**/*.{js,ts,jsx,tsx}'],
  theme: {
    extend: {
      colors: {
        // Nostr Nations color palette
        primary: {
          DEFAULT: '#1a365d',
          50: '#e8f4fc',
          100: '#c5e0f7',
          200: '#9dc9f0',
          300: '#74b0e8',
          400: '#4a98e0',
          500: '#2680d8',
          600: '#1a5da8',
          700: '#1a365d',
          800: '#142847',
          900: '#0e1a30',
        },
        secondary: {
          DEFAULT: '#d69e2e',
          50: '#fdf8e8',
          100: '#faefc5',
          200: '#f6e39d',
          300: '#f1d674',
          400: '#ebca4c',
          500: '#d69e2e',
          600: '#b07d24',
          700: '#8a5c1a',
          800: '#643c10',
          900: '#3e1c06',
        },
        background: {
          DEFAULT: '#1a202c',
          light: '#2d3748',
          lighter: '#4a5568',
        },
        foreground: {
          DEFAULT: '#f7fafc',
          muted: '#a0aec0',
          dim: '#718096',
        },
        success: '#48bb78',
        danger: '#f56565',
        warning: '#ed8936',
        // Player colors
        player: {
          blue: '#3182ce',
          red: '#e53e3e',
          green: '#38a169',
          yellow: '#d69e2e',
        },
      },
      fontFamily: {
        header: ['Cinzel', 'serif'],
        body: ['Inter', 'sans-serif'],
        mono: ['JetBrains Mono', 'monospace'],
      },
      animation: {
        'fade-in': 'fadeIn 0.3s ease-in-out',
        'slide-up': 'slideUp 0.3s ease-out',
        'pulse-gold': 'pulseGold 2s infinite',
        'slide-in-right': 'slideInRight 0.3s ease-out',
        'slide-out-right': 'slideOutRight 0.3s ease-in',
        'scale-in': 'scaleIn 0.2s ease-out',
      },
      keyframes: {
        fadeIn: {
          '0%': { opacity: '0' },
          '100%': { opacity: '1' },
        },
        slideUp: {
          '0%': { transform: 'translateY(10px)', opacity: '0' },
          '100%': { transform: 'translateY(0)', opacity: '1' },
        },
        pulseGold: {
          '0%, 100%': { boxShadow: '0 0 0 0 rgba(214, 158, 46, 0.4)' },
          '50%': { boxShadow: '0 0 0 8px rgba(214, 158, 46, 0)' },
        },
        slideInRight: {
          '0%': { transform: 'translateX(100%)', opacity: '0' },
          '100%': { transform: 'translateX(0)', opacity: '1' },
        },
        slideOutRight: {
          '0%': { transform: 'translateX(0)', opacity: '1' },
          '100%': { transform: 'translateX(100%)', opacity: '0' },
        },
        scaleIn: {
          '0%': { transform: 'scale(0.95)', opacity: '0' },
          '100%': { transform: 'scale(1)', opacity: '1' },
        },
      },
    },
  },
  plugins: [],
}
