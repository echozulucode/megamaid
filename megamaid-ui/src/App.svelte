<script lang="ts">
  import Router from 'svelte-spa-router';
  import Layout from './lib/components/Layout.svelte';

  // Route components
  import Home from './lib/routes/Home.svelte';
  import Scan from './lib/routes/Scan.svelte';
  import Results from './lib/routes/Results.svelte';
  import Plan from './lib/routes/Plan.svelte';
  import Execute from './lib/routes/Execute.svelte';

  const routes = {
    '/': Home,
    '/scan': Scan,
    '/results': Results,
    '/plan': Plan,
    '/execute': Execute,
  };
</script>

<Layout>
  <Router {routes} />
</Layout>
