<script>
  import '../app.css';
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { initUiScale, initLang } from '$lib/stores/settings';

  let { children } = $props();

  onMount(() => {
    invoke('write_session_log', { level: 'info', message: 'Frontend layout mounted' }).catch(() => {});

    const onError = (event) => {
      invoke('write_session_log', {
        level: 'error',
        message: `Uncaught JS error: ${event.message || event.error}`,
      }).catch(() => {});
    };
    const onRejection = (event) => {
      invoke('write_session_log', {
        level: 'error',
        message: `Unhandled promise rejection: ${event.reason}`,
      }).catch(() => {});
    };
    window.addEventListener('error', onError);
    window.addEventListener('unhandledrejection', onRejection);

    // Restore UI scale and language from localStorage (before config loads)
    initUiScale();
    initLang();

    return () => {
      window.removeEventListener('error', onError);
      window.removeEventListener('unhandledrejection', onRejection);
    };
  });
</script>

{@render children?.()}
