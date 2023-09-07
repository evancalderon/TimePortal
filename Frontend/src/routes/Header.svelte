<script lang="ts">
	import { onMount } from 'svelte';
	import { online } from '$lib/global';

	$: time = set_time();

	onMount(() => {
		const interval = setInterval(set_time, 10000);

		return () => {
			clearInterval(interval);
		};
	});

	function set_time(): string {
		return (time = new Date().toLocaleTimeString(undefined, {
			year: 'numeric',
			month: 'numeric',
			day: 'numeric',
			hour: '2-digit',
			minute: '2-digit',
		}));
	}
</script>

<header>
	<h1 class="time">{time}</h1>
	{#if !$online}
		<h6>
			It looks like there was a problem,
			<a href="/" on:click={() => window.location.reload()}>try reloading</a>
		</h6>
	{/if}
</header>

<style lang="scss">
	header {
		display: flex;
		flex-direction: column;
		justify-content: center;
		text-align: center;
		flex-shrink: 1;

		h1 {
			margin: 6px;
		}

		h6 {
			margin: 0;
		}
	}

	.time {
		font-family: var(--font-mono);
	}
</style>
