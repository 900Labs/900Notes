import { mount } from 'svelte'
import MobileApp from './MobileApp.svelte'
import '../app.css'

const app = mount(MobileApp, {
  target: document.getElementById('app')!,
})

export default app
