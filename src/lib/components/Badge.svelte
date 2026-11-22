<script>
  let {
    variant = 'neutral',
    text = '',
    mono = false,
  } = $props();

  const variantStyles = {
    neutral: 'bg-[#202735] text-[#94a3b8] border-[#313c4f]',
    success: 'bg-[var(--accent-badge-bg)] text-[var(--accent-badge-text)] border-[var(--accent-border)]',
    theme: 'bg-[var(--accent-badge-bg)] text-[var(--accent-badge-text)] border-[var(--accent-border)]',
    warning: 'bg-[#3b2b14] text-[#fcd34d] border-[#f59e0b]/30',
    error: 'bg-[#3b171c] text-[#f87171] border-[#ef4444]/30',
    sdk: 'bg-[#16273f] text-[#60a5fa] border-[#3b82f6]/30',
    modeling: 'bg-[#332014] text-[#fb923c] border-[#f97316]/30',
    inspector: 'bg-[#142e32] text-[#2dd4bf] border-[#14b8a6]/30',
    packer: 'bg-[var(--accent-badge-bg)] text-[var(--accent-badge-text)] border-[var(--accent-border)]',
  };
</script>

<span
  class="inline-flex items-center px-1.5 py-0.5 text-[10px] font-semibold border rounded leading-tight select-none
    {mono ? 'font-mono' : 'font-sans'}
    {variantStyles[variant] || variantStyles.neutral}"
>
  {text}
</span>
