module Instruction (
  Instruction(..)
) where

import Prelude

import Control.Monad.Error.Class (throwError)
import Data.Argonaut.Decode (decodeJson, (.:))
import Data.Argonaut.Decode.Class (class DecodeJson)
import Data.Argonaut.Decode.Error (JsonDecodeError(..))
import Data.Argonaut.Encode (encodeJson)
import Data.Argonaut.Encode.Class (class EncodeJson)

data Instruction
  = Stop
  | Play String
  | Pause
  | Resume

instance DecodeJson Instruction where
  decodeJson j = do
    o <- decodeJson j
    tag <- o .: "tag"
    case tag of
      "Stop" ->
        pure Stop
      "Play" ->
        Play <$> o .: "path"
      "Pause" ->
        pure Pause
      "Resume" ->
        pure Resume
      _ ->
        throwError (UnexpectedValue $ encodeJson tag)

instance EncodeJson Instruction where
  encodeJson = case _ of
    Stop ->
      encodeJson { tag: "Stop" }
    Play path ->
      encodeJson { tag: "Play", path }
    Pause ->
      encodeJson { tag: "Pause" }
    Resume ->
      encodeJson { tag: "Resume" }
