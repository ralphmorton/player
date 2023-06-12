module Main where

import Prelude hiding (div)

import Control.Monad.Error.Class (catchError)
import Control.Monad.Rec.Class (forever)
import Data.Argonaut.Encode (toJsonString)
import Data.HTTP.Method (Method(..))
import Data.Newtype (wrap)
import Data.Tuple.Nested ((/\))
import Effect (Effect)
import Effect.Aff (Aff, delay, launchAff_)
import Effect.Class (liftEffect)
import Effect.Exception (message)
import Fetch (fetch)
import Fetch.Argonaut.Json (fromJson)
import Instruction (Instruction(..))
import Muon (Html, Muon, Prop, Signal, aff, div, el, muon, state, text, (:=))

type State = {
  path :: String,
  behaviour :: Behaviour
}

type StateChans = {
  path :: String -> Effect Unit,
  behaviour :: Behaviour -> Effect Unit
}

data Behaviour
  = NetworkError String
  | Idle
  | Playing
  | Paused

main :: Effect Unit
main = muon =<< app

app :: Effect (Signal (Muon Html))
app = do
  sig /\ chans <- state { path: "", behaviour: Idle }
  poll chans
  pure $ sig <#> \{ path, behaviour } -> case behaviour of
    NetworkError error ->
      pure $
        div ["class" := "error message"] [text $ "Error: " <> error]
    Idle ->
      pure $
        div ["class" := "info message"] [text "Waiting..."]
    Playing ->
      renderVideo path false
    Paused ->
      renderVideo path true

renderVideo :: String -> Boolean -> Muon Html
renderVideo path paused = do
  aff (waitDOM *> if paused then pause else play)
  pure $
    div ["class" := "wrapper"] [
      video
        ["id" := videoId, "class" := "vid", "autoplay" := "true", "key" := path]
        [source path]
    ]

video :: Array Prop -> Array Html -> Html
video = el "video"

source :: String -> Html
source src = el "source" ["src" := src] []

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

--
-- Network stuff
--

poll :: StateChans -> Effect Unit
poll chans = launchAff_ $ forever do
  update chans
  delay (wrap 250.0)

update :: StateChans -> Aff Unit
update chans = flip catchError (liftEffect <<< chans.behaviour <<< NetworkError <<< message) do
  { json } <- fetch "/update" {
    method: POST,
    body: toJsonString unit,
    headers: {
      "Accept": "application/json",
      "Content-Type": "application/json"
    }
  }
  rsp <- fromJson json
  liftEffect $ case rsp of
    Stop -> do
      chans.path ""
      chans.behaviour Idle
    Play path -> do
      chans.path path
      chans.behaviour Playing
    Pause ->
      chans.behaviour Paused
    Resume ->
      chans.behaviour Playing
