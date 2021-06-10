import Vue from 'vue'
import App from './App.vue'
import '@/assets/main.scss'

import { library } from '@fortawesome/fontawesome-svg-core'
import { faAngleDown, faAngleUp } from '@fortawesome/free-solid-svg-icons'
import { FontAwesomeIcon } from '@fortawesome/vue-fontawesome'

library.add(faAngleDown, faAngleUp)
Vue.component('font-awesome-icon', FontAwesomeIcon)

Vue.config.productionTip = false

new Vue({
  render: h => h(App),
}).$mount('#app')
