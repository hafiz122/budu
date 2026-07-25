import type { Config } from 'tailwindcss';

export default {
  content: ['./index.html', './src/**/*.{js,ts,jsx,tsx}'],
  darkMode: 'class',
  theme: {
    extend: {
      colors: {
        surface: {
          DEFAULT: '#1c1c1e',
          alt: '#242426',
          hover: '#323235',
        },
        accent: {
          DEFAULT: '#0a84ff',
          hover: '#409cff',
          muted: '#0a84ff29',
        },
        text: {
          primary: '#f5f5f7',
          secondary: '#a1a1a6',
          muted: '#6e6e73',
        },
        border: {
          DEFAULT: '#ffffff1a',
          focus: '#0a84ff',
        },
        success: '#30d158',
        warning: '#ff9f0a',
        danger: '#ff453a',
      },
      borderRadius: {
        DEFAULT: '3px',
        sm: '2px',
        lg: '4px',
      },
      fontFamily: {
        sans: ['-apple-system', 'BlinkMacSystemFont', 'SF Pro Text', 'Helvetica Neue', 'sans-serif'],
        mono: ['SFMono-Regular', 'SF Mono', 'Menlo', 'Monaco', 'monospace'],
      },
      fontSize: {
        xs: ['11px', '15px'],
        sm: ['12px', '17px'],
        base: ['13px', '19px'],
        lg: ['15px', '21px'],
        xl: ['20px', '26px'],
      },
    },
  },
  plugins: [],
} satisfies Config;
