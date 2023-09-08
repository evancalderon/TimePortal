<script lang="ts">
	import moment from 'moment';
	import { onMount } from 'svelte';

	export let name: string;
	export let belt: string;
	export let time_start: string;
	export let time_end: string;

	let time_left: number;
	$: time_left = get_time_left();
	
	let colors = ['white', 'yellow', 'orange', 'green', 'blue', 'purple', 'brown', 'red', 'black'];
	let color_map = new Map(colors.map((v) => [v, v]));

	let split = time_end.split(':');
	let hour = split[0];
	let minute_split = split[1].split(' ');
	let minute = minute_split[0];
	let ampm = minute_split[1];
	let moment_end = moment()
		.set('hour', parseInt(hour))
		.set('minute', parseInt(minute));

	if (ampm == "pm") {
		moment_end = moment_end.set('hour', parseInt(hour) % 12 + 12)
	}

	onMount(() => {
		time_left = get_time_left();

		if (time_left == 0) {
			return;
		}

		let timer = setInterval(() => {
			time_left = get_time_left();
			if (time_left == 0) {
				clearInterval(timer);
			}
		}, 15000);

		return () => {
			clearInterval(timer);
		};
	});

	function get_time_left() {
		return Math.max(0, moment_end.diff(moment(), 'minutes') + 1);
	}
</script>

<div class="card">
	<div>
		<div class="circle">
			<div class="circle" style="border: 10px solid {color_map.get(belt) || 'white'};">
				<div class="circle" />
			</div>
		</div>
		<div>
			<h3>{name}</h3>
			<h5>{belt}</h5>
		</div>
	</div>
	<div>
		<div style="display: grid">
			<h1 style="color: {time_left <= 5 ? 'red' : 'white'}">{time_left}</h1>
		</div>
		<div>
			<p>{time_start}-</p>
			<p>{time_end}</p>
		</div>
	</div>
</div>

<style lang="scss">
	.card {
		display: flex;
		flex-direction: column;
		border-radius: 1.5em;
		padding: 0.5em;
		width: 15em;

		background-color: color-mix(in srgb, var(--color-bg-0) 50%, transparent);
		backdrop-filter: blur(16px);

		> div {
			display: flex;
			flex-direction: row;
			gap: 0.5em;
			justify-content: space-between;
			align-content: center;

			> .circle {
				height: 2em;
				width: 2em;
			}
		}

		h1,
		h3,
		h5,
		p {
			margin: 0 auto;
			margin-right: 0.5em;
			text-align: center;
			align-self: center;
			width: 100%;
		}

		p:first-of-type {
			margin-top: 0.5em;
		}
	}

	.circle {
		border: 2px solid var(--color-text);
		border-radius: 100%;
	}
	
	.circle::before {
		float: left;
		padding-top: 0.5em;
		content: '';
	}

	.circle::after {
		display: block;
		content: '';
		clear: both;
	}
</style>
