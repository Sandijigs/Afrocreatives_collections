/** @type {import('tailwindcss').Config} */
module.exports = {
  content: [
    './src/pages/**/*.{js,ts,jsx,tsx,mdx}',
    './src/components/**/*.{js,ts,jsx,tsx,mdx}',
    './src/app/**/*.{js,ts,jsx,tsx,mdx}',
  ],
  theme: {
    extend: {
      colors: {
        // African/Wakanda inspired color palette
        african: {
          gold: '#FFD700',
          bronze: '#CD7F32',
          copper: '#B87333',
          earth: '#8B4513',
          sunset: '#FF6B35',
          dawn: '#F7931E',
          forest: '#228B22',
          savanna: '#DAA520',
          royal: '#4B0082',
          night: '#1a1a1a',
          warm: '#FFA500',
        },
        wakanda: {
          purple: {
            50: '#f3f1ff',
            100: '#ebe5ff',
            200: '#d9ceff',
            300: '#bea6ff',
            400: '#9f75ff',
            500: '#843dff',
            600: '#7916ff',
            700: '#6b04fd',
            800: '#5a03d4',
            900: '#4B0082',
            950: '#2e0055',
          },
          gold: {
            50: '#fffbeb',
            100: '#fef3c7',
            200: '#fde68a',
            300: '#fcd34d',
            400: '#fbbf24',
            500: '#f59e0b',
            600: '#d97706',
            700: '#b45309',
            800: '#92400e',
            900: '#78350f',
            950: '#451a03',
          },
        },
        primary: {
          50: '#f3f1ff',
          100: '#ebe5ff',
          200: '#d9ceff',
          300: '#bea6ff',
          400: '#9f75ff',
          500: '#843dff',
          600: '#7916ff',
          700: '#6b04fd',
          800: '#5a03d4',
          900: '#4B0082',
          950: '#2e0055',
        },
      },
      backgroundImage: {
        'african-pattern': "url('/patterns/african-pattern.svg')",
        'wakanda-tech': "url('/patterns/wakanda-tech.svg')",
        'gradient-radial': 'radial-gradient(var(--tw-gradient-stops))',
        'gradient-conic': 'conic-gradient(from 180deg at 50% 50%, var(--tw-gradient-stops))',
      },
      fontFamily: {
        'african': ['Exo 2', 'Inter', 'sans-serif'],
        'heading': ['Orbitron', 'Inter', 'sans-serif'],
      },
      animation: {
        'float': 'float 6s ease-in-out infinite',
        'glow': 'glow 2s ease-in-out infinite alternate',
        'fadeIn': 'fadeIn 0.5s ease-in-out',
        'slideUp': 'slideUp 0.5s ease-out',
      },
      keyframes: {
        float: {
          '0%, 100%': { transform: 'translateY(0px)' },
          '50%': { transform: 'translateY(-20px)' },
        },
        glow: {
          '0%': { boxShadow: '0 0 5px #7916ff, 0 0 10px #7916ff, 0 0 15px #7916ff' },
          '100%': { boxShadow: '0 0 10px #7916ff, 0 0 20px #7916ff, 0 0 30px #7916ff' },
        },
        fadeIn: {
          '0%': { opacity: '0' },
          '100%': { opacity: '1' },
        },
        slideUp: {
          '0%': { transform: 'translateY(100px)', opacity: '0' },
          '100%': { transform: 'translateY(0)', opacity: '1' },
        },
      },
    },
  },
  plugins: [],
}