export const play_ = (id) => () => document.getElementById(id).play()

export const pause_ = (id) => () => document.getElementById(id).pause()

export const playerState_ = (id) => () => {
  const video = document.getElementById(id)

  if (video && video.children[0] && video.children[0].src && video.duration && video.currentTime) {
    const path = video.children[0].getAttribute("meta-path")

    return {
      path,
      duration: video.duration,
      time: video.currentTime
    }
  } else {
    return null
  }
}

export const setCurrentTime_ = (id) => (time) => () => {
  const video = document.getElementById(id)

  if (video && video.currentTime) {
    video.currentTime = time
  }
}
