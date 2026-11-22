<script>
  let {
    variant = 'tonal', // 'filled' | 'tonal' | 'outlined' | 'destructive' | 'ghost'
    size = 'md',        // 'sm' | 'md'
    disabled = false,
    title = '',
    type = 'button',
    isFullWidth = false,
    ariaLabel = '',
    onclick = undefined,
    children
  } = $props();

  const sizeStyles = {
    sm: 'h-7 min-h-[28px] px-2.5 text-xs rounded gap-1.5 font-medium',
    md: 'h-8 min-h-[32px] px-3.5 text-xs rounded gap-2 font-medium',
  };
</script>

<button
  {type}
  {title}
  disabled={disabled}
  aria-disabled={disabled ? 'true' : undefined}
  aria-label={ariaLabel || undefined}
  onclick={onclick}
  style={variant === 'filled' ? 'background-color: var(--accent-primary); color: var(--accent-on-primary);' : undefined}
  class="inline-flex items-center justify-center select-none transition-all duration-150 rounded-[5px]
    {variant === 'filled' ? 'hover:brightness-110 active:brightness-95 font-semibold shadow-xs' : ''}
    {variant === 'tonal' ? 'bg-[#202633] hover:bg-[#283142] active:bg-[#1b202b] text-[#e2e8f0] border border-[#313c4f]' : ''}
    {variant === 'outlined' ? 'bg-transparent hover:bg-[#1a212d] active:bg-[#202735] border border-[#364257] text-[#e2e8f0]' : ''}
    {variant === 'destructive' ? 'bg-[#381a1e] hover:bg-[#482025] active:bg-[#2e1518] text-[#fca5a5] border border-[#5e222a]' : ''}
    {variant === 'ghost' ? 'bg-transparent hover:bg-[#1a202c] active:bg-[#222938] text-[#94a3b8] hover:text-[#f1f5f9]' : ''}
    {sizeStyles[size] || sizeStyles.md}
    {isFullWidth ? 'w-full' : ''}
    {disabled ? 'opacity-40 cursor-not-allowed pointer-events-none' : 'cursor-pointer active:scale-[0.98]'}"
>
  {@render children?.()}
</button>
