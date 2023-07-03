module Main where

import Prelude hiding (div)

import Control.Monad.Error.Class (catchError)
import Control.Monad.Rec.Class (forever)
import Data.Argonaut.Encode (toJsonString)
import Data.Foldable (traverse_)
import Data.HTTP.Method (Method(..))
import Data.Newtype (wrap)
import Data.Maybe (Maybe(..))
import Data.Nullable (Nullable, toMaybe)
import Data.Tuple.Nested ((/\))
import Effect (Effect)
import Effect.Aff (Aff, delay, launchAff_)
import Effect.Class (liftEffect)
import Effect.Exception (message)
import Fetch (fetch)
import Fetch.Argonaut.Json (fromJson)
import Instruction (Behaviour(..), Instruction(..))
import Muon (Html, Muon, Prop, Signal, aff, div, el, muon, state, text, (:=))
import PlayerState (PlayerState)

type State = {
  action :: Action
}

type StateChans = {
  action :: Action -> Effect Unit
}

data Action
  = NetworkError String
  | Waiting
  | Video String Behaviour

main :: Effect Unit
main = muon =<< app

app :: Effect (Signal (Muon Html))
app = do
  sig /\ chans <- state { action: Waiting }
  poll chans
  pure $ sig <#> \{ action } -> case action of
    NetworkError err ->
      pure $
        div ["class" := "error message"] [text err]
    Waiting ->
      pure $
        div ["class" := "info message"] [text "Waiting..."]
    Video path behaviour ->
      renderVideo path case behaviour of
        Playing -> false
        Paused -> true

renderVideo :: String -> Boolean -> Muon Html
renderVideo path paused = do
  aff (waitDOM *> if paused then pause else play)
  pure $
    div ["class" := "wrapper"] [
      video
        ["id" := videoId, "class" := "vid", "autoplay" := "true", "key" := path]
        [el "source" ["src" := ("/play/" <> path), "meta-path" := path] []]
    ]

video :: Array Prop -> Array Html -> Html
video = el "video"

videoId :: String
videoId = "vid"

waitDOM :: Aff Unit
waitDOM = delay (wrap 50.0)

play :: Aff Unit
play = liftEffect $ play_ videoId

foreign import play_ :: String -> Effect Unit

pause :: Aff Unit
pause = liftEffect $ pause_ videoId

foreign import pause_ :: String -> Effect Unit

playerState :: Aff (Maybe PlayerState)
playerState = toMaybe <$> liftEffect (playerState_ videoId)

foreign import playerState_ :: String -> Effect (Nullable PlayerState)

setCurrentTime :: Number -> Effect Unit
setCurrentTime = setCurrentTime_ videoId

foreign import setCurrentTime_ :: String -> Number -> Effect Unit

--
-- Network stuff
--

poll :: StateChans -> Effect Unit
poll chans = launchAff_ $ forever do
  update chans
  delay (wrap 250.0)

update :: StateChans -> Aff Unit
update chans = flip catchError (liftEffect <<< chans.action <<< NetworkError <<< message) do
  pState <- playerState
  { json } <- fetch "/update" {
    method: POST,
    body: toJsonString pState,
    headers: {
      "Accept": "application/json",
      "Content-Type": "application/json"
    }
  }
  rsp <- fromJson json
  liftEffect case rsp of
    Just Idle ->
      chans.action Waiting
    Just (Play path from behaviour) -> do
      chans.action (Video path behaviour)
      traverse_ setCurrentTime from
    Nothing ->
      pure unit
