import Vue from 'vue'
import App from './App.vue'
import '@/assets/main.scss'

import { library } from '@fortawesome/fontawesome-svg-core'
import { faAngleDown, faAngleUp, faCog, faList } from '@fortawesome/free-solid-svg-icons'
import { FontAwesomeIcon} from '@fortawesome/vue-fontawesome'

library.add(faAngleDown, faAngleUp, faCog, faList )
Vue.component('font-awesome-icon', FontAwesomeIcon)

Vue.config.productionTip = false

import devtools from '@vue/devtools'
if (process.env.NODE_ENV === 'development') {
  devtools.connect(/* host, port */)
}

new Vue({
  render: h => h(App),
}).$mount('#app')
