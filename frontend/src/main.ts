import './assets/main.css'
import 'primeicons/primeicons.css'

import { createApp } from 'vue'
import App from './App.vue'
import PrimeVue from 'primevue/config';
import Aura from '@primeuix/themes/aura';
import Tooltip from 'primevue/tooltip';
import ToastService from 'primevue/toastservice';

const app = createApp(App)
app.use(PrimeVue, { theme: { preset: Aura } })

app.use(ToastService);
app.directive('tooltip', Tooltip);

app.mount('#app')
