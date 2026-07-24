import type { Config } from 'tailwindcss';

export default {
  content: ['./index.html', './src/**/*.{js,ts,jsx,tsx}'],
  darkMode: 'class',
  theme: {
    extend: {
      colors: {
        surface: {
          DEFAULT: '#3c3c3c',
          alt: '#2d2d2d',
          hover: '#4a4a4a',
        },
        accent: {
          DEFAULT: '#7c9c2e',
          hover: '#8db834',
          muted: '#7c9c2e33',
        },
        text: {
          primary: '#e0e0d0',
          secondary: '#a0a090',
          muted: '#707060',
        },
        border: {
          DEFAULT: '#555555',
          focus: '#7c9c2e',
        },
        success: '#5a8f3c',
        warning: '#c49c30',
        danger: '#b33a3a',
      },
      borderRadius: {
        DEFAULT: '3px',
        sm: '2px',
        lg: '4px',
      },
      fontFamily: {
        sans: ['Tahoma', 'Geneva', 'Verdana', 'sans-serif'],
        mono: ['Courier New', 'Courier', 'monospace'],
      },
      fontSize: {
        xs: ['10px', '14px'],
        sm: ['11px', '16px'],
        base: ['12px', '18px'],
        lg: ['13px', '20px'],
        xl: ['14px', '22px'],
      },
    },
  },
  plugins: [],
} satisfies Config;
