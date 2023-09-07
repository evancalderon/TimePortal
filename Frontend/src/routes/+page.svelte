<script lang="ts">
	import { onMount } from 'svelte';
	import Card from './Card.svelte';
	import { online } from '$lib/global';

	type Student = {
		name: string;
		belt: string;
		time_start: string;
		time_end: string;
	};

	let students: Array<Student> = [];
	$: students;
	onMount(async () => {
		fetch('/api/forcerefresh', {
			method: 'POST',
		});
		students = await fetch_students();
		setInterval(fetch_students, 15 * 1000);
	});
	async function fetch_students() {
		try {
			students = (await (await fetch('/api/students')).json()) as Array<Student>;
			$online = true;
			return students;
		} catch {
			$online = false;
		}
		return [];
	}
</script>

<svelte:head>
	<title>Home</title>
	<meta name="description" content="Svelte demo app" />
</svelte:head>

<section>
	{#each students as student}
		<Card
			belt={student.belt.split(' ')[0].toLowerCase()}
			name={student.name}
			time_start={student.time_start}
			time_end={student.time_end}
		/>
	{/each}
</section>

<style>
	section {
		display: flex;
		flex-direction: row;
		flex-wrap: wrap;
		justify-content: center;
		align-items: center;
		gap: 1em;
	}
</style>
