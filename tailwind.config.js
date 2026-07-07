/** @type {import('tailwindcss').Config} */
export default {
  content: ['./index.html', './src/**/*.{svelte,ts,js}'],
  darkMode: 'class',
  theme: {
    extend: {
      colors: {
        accent: {
          DEFAULT: '#2f6fce',
          light: '#6ea0ff',
          hover: '#255db7',
          dark: '#1e4fa8',
          soft: '#dbe8ff',
        },
        gray: {
          50: '#f7f5f2',
          100: '#eeeae4',
          200: '#ded8cf',
          300: '#c6beb3',
          400: '#958d84',
          500: '#6b655d',
          600: '#4b4742',
          700: '#242728',
          800: '#18191a',
          900: '#0f1011',
          950: '#07080a',
        },
      },
      fontFamily: {
        sans: ['Inter', 'ui-sans-serif', 'system-ui', '-apple-system', 'BlinkMacSystemFont', 'Segoe UI', 'sans-serif'],
        mono: ['ui-monospace', 'Consolas', 'monospace'],
      },
    },
  },
  plugins: [],
}
