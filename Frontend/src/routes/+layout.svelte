<script>
	import Header from './Header.svelte';
	import './styles.scss';
</script>

<div class="app">
	<Header />

	<main>
		<slot />
	</main>
</div>

<style>
	.app {
		display: flex;
		flex-direction: column;
		min-height: 100vh;
	}

	main {
		display: flex;
		flex-direction: column;
		padding: 1rem;
		width: 100%;
		max-width: 80%;
		margin: 0 auto;
		box-sizing: border-box;
		flex-grow: 1;
		justify-content: center;
	}
</style>
