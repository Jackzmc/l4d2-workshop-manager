import Vue from 'vue'
import App from './App.vue'
import '@/assets/main.scss'

import Buefy from 'buefy'
import 'buefy/dist/buefy.css'
Vue.use(Buefy, {
  defaultIconPack: 'fas',
  defaultIconComponent: "font-awesome-icon",
})

import { library } from '@fortawesome/fontawesome-svg-core'
import { faAngleDown, faAngleUp, faCog, faList, faWindowMinimize, faTimes, faExclamationTriangle, faPlus, faSearch, faLink, faArrowUp, faAngleRight, faLongArrowAltLeft } from '@fortawesome/free-solid-svg-icons'
import { faWindowMaximize } from '@fortawesome/free-regular-svg-icons'
import { FontAwesomeIcon } from '@fortawesome/vue-fontawesome'

library.add(faAngleDown, faAngleUp, faCog, faList, faWindowMinimize, faWindowMaximize, faTimes, faExclamationTriangle, faPlus, faSearch, faLink, faAngleRight, faArrowUp, faLongArrowAltLeft)
Vue.component('font-awesome-icon', FontAwesomeIcon)

Vue.config.productionTip = false

Vue.prototype.$VERSION = process.env.VUE_APP_VERSION
Vue.prototype.$BUILD   = process.env.VUE_APP_BUILD

import devtools from '@vue/devtools'
if (process.env.NODE_ENV === 'development') {
  devtools.connect(/* host, port */)
}

new Vue({
  render: h => h(App),
}).$mount('#app')
