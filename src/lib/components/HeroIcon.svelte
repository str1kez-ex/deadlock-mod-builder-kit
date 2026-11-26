<!-- HeroIcon.svelte — Unified hero icon component -->
<script lang="ts">
  import { getHeroIcon } from '$lib/constants/heroes';

  interface Props {
    name: string;
    size?: number;
    class?: string;
  }

  let { name, size = 32, class: cls = '' }: Props = $props();

  const icon = $derived(getHeroIcon(name));
  const initials = $derived(
    name
      .split(' ')
      .map((w) => w[0])
      .join('')
      .slice(0, 2)
      .toUpperCase()
  );
</script>

{#if icon}
  <img
    src={icon}
    alt={name}
    width={size}
    height={size}
    class="rounded-full object-cover {cls}"
    style="width:{size}px;height:{size}px"
    loading="lazy"
  />
{:else}
  <div
    class="rounded-full flex items-center justify-center bg-gray-700 text-xs font-bold text-white {cls}"
    style="width:{size}px;height:{size}px;font-size:{Math.max(8, size * 0.35)}px"
    title={name}
  >
    {initials}
  </div>
{/if}
