/** @type {import('tailwindcss').Config} */
export default {
  content: ['./src/**/*.{html,js,svelte,ts}'],
  theme: {
    extend: {
      colors: {
        dmb: {
          darkest: '#0c0e12',
          sidebar: '#12151b',
          main: '#151922',
          card: '#1a1f2a',
          'card-hover': '#212735',
          input: '#101319',
          border: '#262f3f',
          'border-light': '#364257',
          primary: '#2ea069',
          'primary-hover': '#36b879',
          'primary-dark': '#1e6b45',
          'text-primary': '#f1f5f9',
          'text-secondary': '#94a3b8',
          'text-muted': '#64748b',
          accent: '#34d399',
        },
      },
      fontFamily: {
        sans: ['Segoe UI', 'Inter', '-apple-system', 'BlinkMacSystemFont', 'system-ui', 'sans-serif'],
        mono: ['JetBrains Mono', 'Consolas', 'monospace'],
      },
      borderRadius: {
        dmb: '6px',
      },
    },
  },
  plugins: [],
};
